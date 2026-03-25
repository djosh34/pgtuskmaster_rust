use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    time::Duration,
};

use crate::support::{
    docker::cli::DockerCli,
    error::{HarnessError, Result},
};

const RYUK_IMAGE: &str = "testcontainers/ryuk:0.14.0";
const RYUK_CONTAINER_PORT: &str = "8080/tcp";
const RYUK_ACK_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct RyukGuard {
    docker: DockerCli,
    container_id: String,
    stream: Option<TcpStream>,
}

impl RyukGuard {
    pub fn start(docker: DockerCli, compose_project: &str) -> Result<Self> {
        let container_id = docker.run_detached(
            vec![
                "run".to_string(),
                "--detach".to_string(),
                "--rm".to_string(),
                "--publish".to_string(),
                "127.0.0.1::8080".to_string(),
                "--volume".to_string(),
                "/var/run/docker.sock:/var/run/docker.sock".to_string(),
                RYUK_IMAGE.to_string(),
            ],
            "starting ryuk sidecar",
        )?;
        let container_id = container_id.trim().to_string();
        if container_id.is_empty() {
            return Err(HarnessError::message(
                "docker run returned an empty container id for ryuk",
            ));
        }

        let host_port = wait_for_host_port(&docker, container_id.as_str())?;
        let stream = wait_for_registration(host_port, compose_project)?;

        Ok(Self {
            docker,
            container_id,
            stream: Some(stream),
        })
    }

    pub fn close(&mut self) -> Result<()> {
        self.stream.take();
        self.docker.sleep_for_resource_cleanup();
        match self
            .docker
            .remove_container_force(self.container_id.as_str())
        {
            Ok(()) => Ok(()),
            Err(HarnessError::CommandFailed { stderr, .. })
                if ryuk_removal_already_completed(stderr.as_str()) =>
            {
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

fn ryuk_removal_already_completed(stderr: &str) -> bool {
    let normalized = stderr.to_ascii_lowercase();
    normalized.contains("removal of container") && normalized.contains("already in progress")
        || normalized.contains("no such container")
}

fn wait_for_host_port(docker: &DockerCli, container_id: &str) -> Result<u16> {
    let mut attempts = 0_u32;
    while attempts < 20 {
        if let Ok(host_port) = docker.published_host_port(container_id, RYUK_CONTAINER_PORT) {
            return Ok(host_port);
        }
        attempts += 1;
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    Err(HarnessError::message(
        "timed out waiting for ryuk to publish a host port",
    ))
}

fn wait_for_tcp_stream(port: u16) -> Result<TcpStream> {
    let address = format!("127.0.0.1:{port}");
    let mut attempts = 0_u32;
    while attempts < 40 {
        match TcpStream::connect(address.as_str()) {
            Ok(stream) => return Ok(stream),
            Err(_) => {
                attempts += 1;
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        }
    }

    Err(HarnessError::message(format!(
        "timed out waiting for ryuk TCP listener at {address}",
    )))
}

fn wait_for_registration(port: u16, compose_project: &str) -> Result<TcpStream> {
    let mut attempts = 0_u32;
    let mut last_error = None;
    while attempts < 20 {
        match try_register_filter(port, compose_project) {
            Ok(stream) => return Ok(stream),
            Err(err) => {
                last_error = Some(err.to_string());
                attempts += 1;
                std::thread::sleep(Duration::from_millis(250));
            }
        }
    }

    Err(HarnessError::message(format!(
        "timed out waiting for ryuk registration acknowledgement on tcp://127.0.0.1:{port}; last observed error: {}",
        last_error.unwrap_or_else(|| "registration did not report an error".to_string())
    )))
}

fn try_register_filter(port: u16, compose_project: &str) -> Result<TcpStream> {
    let mut stream = wait_for_tcp_stream(port)?;
    let filter_line = format!("label=com.docker.compose.project={compose_project}\n");
    let path = std::path::PathBuf::from(format!("tcp://127.0.0.1:{port}"));
    stream
        .write_all(filter_line.as_bytes())
        .map_err(|source| HarnessError::Io {
            path: path.clone(),
            source,
        })?;
    stream.flush().map_err(|source| HarnessError::Io {
        path: path.clone(),
        source,
    })?;
    wait_for_registration_ack(&stream, port)?;
    Ok(stream)
}

fn wait_for_registration_ack(stream: &TcpStream, port: u16) -> Result<()> {
    let path = std::path::PathBuf::from(format!("tcp://127.0.0.1:{port}"));
    stream
        .set_read_timeout(Some(RYUK_ACK_TIMEOUT))
        .map_err(|source| HarnessError::Io {
            path: path.clone(),
            source,
        })?;
    let mut reader = BufReader::new(stream.try_clone().map_err(|source| HarnessError::Io {
        path: path.clone(),
        source,
    })?);
    let mut ack_line = String::new();
    let bytes_read = reader
        .read_line(&mut ack_line)
        .map_err(|source| HarnessError::Io {
            path: path.clone(),
            source,
        })?;
    stream
        .set_read_timeout(None)
        .map_err(|source| HarnessError::Io { path, source })?;
    if bytes_read == 0 {
        return Err(HarnessError::message(
            "ryuk closed the registration socket before acknowledging the filter",
        ));
    }

    validate_registration_ack(ack_line.trim())
}

fn validate_registration_ack(ack_line: &str) -> Result<()> {
    if ack_line == "ACK" {
        Ok(())
    } else {
        Err(HarnessError::message(format!(
            "ryuk returned an unexpected registration acknowledgement `{ack_line}`",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_registration_ack;

    #[test]
    fn registration_ack_accepts_ack() -> Result<(), String> {
        validate_registration_ack("ACK").map_err(|err| err.to_string())
    }

    #[test]
    fn registration_ack_rejects_non_ack() -> Result<(), String> {
        match validate_registration_ack("PONG") {
            Ok(()) => Err("expected unexpected ryuk acknowledgement to fail".to_string()),
            Err(err) => {
                let message = err.to_string();
                if message.contains("unexpected registration acknowledgement") {
                    Ok(())
                } else {
                    Err(format!("unexpected error message: {message}"))
                }
            }
        }
    }
}
