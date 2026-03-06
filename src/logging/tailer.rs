use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tokio::io::{AsyncReadExt, AsyncSeekExt};

use crate::state::WorkerError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StartPosition {
    Beginning,
    End,
}

#[derive(Clone, Debug)]
pub(crate) struct FileTailer {
    path: PathBuf,
    start: StartPosition,
    offset: Option<u64>,
    pending: Vec<u8>,
    #[cfg(unix)]
    last_inode: Option<u64>,
}

impl FileTailer {
    pub(crate) fn new(path: PathBuf, start: StartPosition) -> Self {
        Self {
            path,
            start,
            offset: None,
            pending: Vec::new(),
            #[cfg(unix)]
            last_inode: None,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub(crate) async fn read_new_lines(
        &mut self,
        max_bytes: usize,
    ) -> Result<Vec<Vec<u8>>, WorkerError> {
        if max_bytes == 0 {
            return Ok(Vec::new());
        }

        let meta = match tokio::fs::metadata(&self.path).await {
            Ok(meta) => meta,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                self.offset = None;
                self.pending.clear();
                #[cfg(unix)]
                {
                    self.last_inode = None;
                }
                return Ok(Vec::new());
            }
            Err(err) => {
                return Err(WorkerError::Message(format!(
                    "tailer metadata failed for {}: {err}",
                    self.path.display()
                )));
            }
        };

        #[cfg(unix)]
        let inode = Some(std::os::unix::fs::MetadataExt::ino(&meta));

        #[cfg(not(unix))]
        let inode: Option<u64> = None;

        let len = meta.len();

        let should_reset_for_rotation = {
            #[cfg(unix)]
            {
                if let (Some(prev), Some(now)) = (self.last_inode, inode) {
                    prev != now
                } else {
                    false
                }
            }
            #[cfg(not(unix))]
            {
                false
            }
        };

        if should_reset_for_rotation {
            self.offset = None;
            self.pending.clear();
        }

        let offset = match self.offset {
            Some(offset) => {
                if len < offset {
                    // truncation
                    0
                } else {
                    offset
                }
            }
            None => match self.start {
                StartPosition::Beginning => 0,
                StartPosition::End => len,
            },
        };

        let mut file = tokio::fs::File::open(&self.path).await.map_err(|err| {
            WorkerError::Message(format!("open failed for {}: {err}", self.path.display()))
        })?;
        file.seek(std::io::SeekFrom::Start(offset))
            .await
            .map_err(|err| {
                WorkerError::Message(format!("seek failed for {}: {err}", self.path.display()))
            })?;

        let mut out = Vec::new();
        let mut read_total = 0usize;
        let mut buf = vec![0u8; 8192];

        while read_total < max_bytes {
            let budget = max_bytes.saturating_sub(read_total);
            let chunk_len = buf.len().min(budget);
            let n = file.read(&mut buf[..chunk_len]).await.map_err(|err| {
                WorkerError::Message(format!("read failed for {}: {err}", self.path.display()))
            })?;
            if n == 0 {
                break;
            }
            read_total = read_total.saturating_add(n);
            self.pending.extend_from_slice(&buf[..n]);

            while let Some(pos) = self.pending.iter().position(|b| *b == b'\n') {
                let mut line = self.pending.drain(..=pos).collect::<Vec<u8>>();
                if let Some(b'\n') = line.last() {
                    line.pop();
                }
                if let Some(b'\r') = line.last() {
                    line.pop();
                }
                out.push(line);
            }
        }

        let new_offset = offset.saturating_add(read_total as u64);
        self.offset = Some(new_offset);
        #[cfg(unix)]
        {
            self.last_inode = inode;
        }
        Ok(out)
    }
}

#[derive(Default)]
pub(crate) struct DirTailers {
    tailers: BTreeMap<PathBuf, FileTailer>,
}

impl DirTailers {
    pub(crate) fn ensure_file(&mut self, path: PathBuf, start: StartPosition) {
        self.tailers
            .entry(path.clone())
            .or_insert_with(|| FileTailer::new(path, start));
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (&PathBuf, &mut FileTailer)> {
        self.tailers.iter_mut()
    }

    pub(crate) fn len(&self) -> usize {
        self.tailers.len()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{FileTailer, StartPosition};

    fn tmp_dir(label: &str) -> PathBuf {
        let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_millis(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!(
            "pgtuskmaster-tailer-{label}-{millis}-{}",
            std::process::id()
        ))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn file_tailer_reads_appends_and_handles_rotation(
    ) -> Result<(), crate::state::WorkerError> {
        let dir = tmp_dir("rotation");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|err| {
            crate::state::WorkerError::Message(format!("create_dir_all failed: {err}"))
        })?;

        let path = dir.join("postgres.log");
        tokio::fs::write(&path, b"a\n")
            .await
            .map_err(|err| crate::state::WorkerError::Message(format!("write failed: {err}")))?;

        let mut tailer = FileTailer::new(path.clone(), StartPosition::Beginning);
        let first = tailer.read_new_lines(1024).await?;
        assert_eq!(first, vec![b"a".to_vec()]);

        tokio::fs::write(&path, b"a\nb\n")
            .await
            .map_err(|err| crate::state::WorkerError::Message(format!("append failed: {err}")))?;
        let second = tailer.read_new_lines(1024).await?;
        assert_eq!(second, vec![b"b".to_vec()]);

        let rotated = dir.join("postgres.log.1");
        tokio::fs::rename(&path, &rotated)
            .await
            .map_err(|err| crate::state::WorkerError::Message(format!("rename failed: {err}")))?;
        tokio::fs::write(&path, b"c\n").await.map_err(|err| {
            crate::state::WorkerError::Message(format!("new file write failed: {err}"))
        })?;

        let third = tailer.read_new_lines(1024).await?;
        assert_eq!(third, vec![b"c".to_vec()]);

        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }
}
