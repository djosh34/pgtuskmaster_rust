use std::{io::Write, net::TcpStream};

use crate::support::{
    docker::cli::DockerCli,
    error::{HarnessError, Result},
};

const RYUK_IMAGE: &str = "testcontainers/ryuk:0.14.0";
const RYUK_CONTAINER_PORT: &str = "8080/tcp";

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
        let mut stream = wait_for_tcp_stream(host_port)?;
        let filter_line = format!("label=com.docker.compose.project={compose_project}\n");
        stream
            .write_all(filter_line.as_bytes())
            .map_err(|source| HarnessError::Io {
                path: std::path::PathBuf::from(format!("tcp://127.0.0.1:{host_port}")),
                source,
            })?;
        stream.flush().map_err(|source| HarnessError::Io {
            path: std::path::PathBuf::from(format!("tcp://127.0.0.1:{host_port}")),
            source,
        })?;

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
