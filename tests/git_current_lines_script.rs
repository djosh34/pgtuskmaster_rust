use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn unique_temp_dir(label: &str) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| format!("system clock before unix epoch: {err}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "pgtm-{label}-{}-{timestamp}",
        std::process::id()
    ));
    match fs::remove_dir_all(&path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(format!("remove existing temp dir {} failed: {err}", path.display()));
        }
    }
    fs::create_dir_all(&path)
        .map_err(|err| format!("create temp dir {} failed: {err}", path.display()))?;
    Ok(path)
}

fn remove_dir_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(format!("remove {} failed: {err}", path.display())),
    }
}

fn with_temp_dir<T>(label: &str, run: impl FnOnce(&Path) -> Result<T, String>) -> Result<T, String> {
    let path = unique_temp_dir(label)?;
    let result = run(path.as_path());
    let cleanup = remove_dir_if_exists(path.as_path());
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(err), Ok(())) => Err(err),
        (Ok(_), Err(cleanup_err)) => Err(cleanup_err),
        (Err(err), Err(cleanup_err)) => Err(format!("{err}\ncleanup also failed: {cleanup_err}")),
    }
}

fn decode_output(bytes: Vec<u8>, stream: &str) -> Result<String, String> {
    String::from_utf8(bytes).map_err(|err| format!("{stream} utf8 decode failed: {err}"))
}

fn run_git(repo_root: &Path, args: &[&str]) -> Result<(), String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()
        .map_err(|err| format!("git {} failed to start: {err}", args.join(" ")))?;
    if output.status.success() {
        return Ok(());
    }

    let stdout = decode_output(output.stdout, "git stdout")?;
    let stderr = decode_output(output.stderr, "git stderr")?;
    Err(format!(
        "git {} failed with {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        args.join(" "),
        output.status.code()
    ))
}

fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|err| format!("write {} failed: {err}", path.display()))
}

fn script_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(".ralph/git_current_lines.sh")
}

#[test]
fn deleted_tracked_files_are_excluded_from_git_current_lines_counts() -> Result<(), String> {
    with_temp_dir("git-current-lines-script", |repo_root| {
        fs::create_dir_all(repo_root.join("src"))
            .map_err(|err| format!("create src dir failed: {err}"))?;
        fs::create_dir_all(repo_root.join("tests"))
            .map_err(|err| format!("create tests dir failed: {err}"))?;

        write_file(&repo_root.join("src/lib.rs"), "pub fn lines() {}\nlet _value = 1;\n")?;
        write_file(&repo_root.join("src/main.rs"), "fn main() {}\n")?;
        write_file(
            &repo_root.join("tests/basic.rs"),
            "#[test]\nfn basic() {}\n\n",
        )?;

        run_git(repo_root, &["init"])?;
        run_git(repo_root, &["add", "src", "tests"])?;
        run_git(
            repo_root,
            &[
                "-c",
                "user.name=Test User",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "initial",
            ],
        )?;

        fs::remove_file(repo_root.join("src/main.rs"))
            .map_err(|err| format!("remove tracked src/main.rs failed: {err}"))?;

        let output = Command::new("/bin/bash")
            .arg(script_path())
            .current_dir(repo_root)
            .output()
            .map_err(|err| format!("failed to run git_current_lines.sh: {err}"))?;

        assert!(
            output.status.success(),
            "script should succeed, got {:?}",
            output.status.code()
        );

        let stdout = decode_output(output.stdout, "stdout")?;
        let stderr = decode_output(output.stderr, "stderr")?;

        assert!(
            !stderr.contains("No such file or directory"),
            "stderr should not report deleted tracked files, got: {stderr}"
        );
        assert_eq!(
            stdout,
            concat!(
                "src/: 2 lines across 1 existing git-tracked files\n",
                "tests/: 3 lines across 1 existing git-tracked files\n",
                "total: 5 lines across 2 existing git-tracked files\n",
            ),
            "stdout should count only the remaining tracked files"
        );

        Ok(())
    })
}
