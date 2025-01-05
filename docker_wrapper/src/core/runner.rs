use crate::shared::error::{AppResult, Error as AppError};
use bollard::container::{
    AttachContainerOptions, AttachContainerResults, Config, RemoveContainerOptions,
};
use bollard::models::{HostConfig, PortBinding, PortMap};
use bollard::Docker;
use futures_util::StreamExt;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::{spawn, time::sleep};

const TIMEOUT_DEFAULT: u64 = 5;
const BYTES_IN_MB: i64 = 1024 * 1024; // 1 MB in bytes
const SIZE_256_MB: i64 = 256 * BYTES_IN_MB; // 256 MB in bytes
const NUM_CPUS: f64 = 1.0;

/// Spawns a Docker container with given image and ports, attaches to it,
/// and sets up a timeout/cleanup mechanism.
///
/// # Arguments
///
/// * `image_name` - Name of the Docker image to run.
/// * `port_binding` - Port mapping string of the form "HOST_PORT:CONTAINER_PORT".
/// * `timeout` - Optional duration after which to trigger a timeout. Defaults to 5s.
///
/// # Returns
///
/// * On success, returns the container ID as a `String`.
/// * On error, returns an `AppError`.
pub async fn runner(
    image_name: &str,
    port_binding: &str,
    timeout: Option<Duration>,
) -> AppResult<String> {
    // Connect to Docker via Unix socket (or named pipe on Windows).
    let docker = Docker::connect_with_socket_defaults()
        .map_err(|e| AppError::System(format!("Failed to connect to Docker: {e}")))?;

    let start_time = Instant::now();

    // Safely parse the port mapping: "HOST_PORT:CONTAINER_PORT".
    let ports: Vec<&str> = port_binding.split(':').collect();
    if ports.len() != 2 {
        return Err(AppError::System(
            "Invalid port binding format; expected 'HOST_PORT:CONTAINER_PORT'.".to_string(),
        ));
    }

    let host_port = ports[0];
    let container_port = ports[1];

    // Set up port bindings.
    let mut port_map = PortMap::new();
    port_map.insert(
        format!("{container_port}/tcp"),
        Some(vec![PortBinding {
            host_ip: Some("127.0.0.1".to_string()),
            host_port: Some(host_port.to_string()),
        }]),
    );

    let (cpu_period, cpu_quota) = cpu_limits(1.0);
    // Configure the container.
    let container_config = Config {
        image: Some(image_name),
        tty: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        host_config: Some(HostConfig {
            memory: Some(SIZE_256_MB),
            cpu_period: Some(cpu_period),
            cpu_quota: Some(cpu_quota),
            port_bindings: Some(port_map),
            auto_remove: Some(true),
            ..Default::default()
        }),
        ..Default::default()
    };

    // Create the container.
    let create_response = docker
        .create_container::<&str, &str>(None, container_config)
        .await
        .map_err(|e| AppError::System(format!("Failed to create container: {e}")))?;
    let container_id = create_response.id.clone();

    // Start the container.
    docker
        .start_container::<String>(&container_id, None)
        .await
        .map_err(|e| AppError::System(format!("Failed to start container: {e}")))?;

    // Attach to the container to retrieve logs (stdout/stderr).
    let AttachContainerResults { mut output, .. } = docker
        .attach_container(
            &container_id,
            Some(AttachContainerOptions::<String> {
                stdout: Some(true),
                stderr: Some(true),
                stream: Some(true),
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| AppError::System(format!("Failed to attach to container: {e}")))?;

    // Optionally, spawn a task to consume and display container logs.
    spawn(async move {
        //TODO: confirm if this auto die once container is killed, if not
        // receive a kill signal for container and stop streaming logs
        while let Some(Ok(logOut)) = output.next().await {
            let bytes = logOut.into_bytes();
            let text = String::from_utf8_lossy(&bytes);
            println!("Container STDOUT: <<< {text} >>>");
        }
    });

    // Spawn a separate task to handle timeout/cleanup.
    let docker_clone = docker.clone();
    let container_id_clone = container_id.clone();
    spawn(async move {
        let timeout_val = timeout.unwrap_or_else(|| Duration::from_secs(TIMEOUT_DEFAULT));

        // Create a channel-based timeout; trigger_timeout() starts the countdown.
        let (rx, trigger_timeout) = crate::shared::utils::timeout(timeout_val);
        trigger_timeout();

        match monitor_container_process(&docker_clone, &container_id_clone, rx).await {
            Ok(_) => {
                let elapsed_time = start_time.elapsed();
                println!(
                    "Execution took {:.2} seconds.",
                    elapsed_time.as_millis() as f64 / 1000.0
                );
            }
            Err(e) => eprintln!("Failed to monitor child process: {e}"),
        }
    });

    Ok(container_id)
}

/// Monitors the container process using a timeout channel.
/// If a message is received, we assume the process completed or timed out,
/// and then we remove the container.
///
/// # Arguments
///
/// * `docker` - Reference to the Docker client.
/// * `container_id` - ID of the running container.
/// * `timeout_rx` - A channel receiver for timeout signals.
async fn monitor_container_process(
    docker: &Docker,
    container_id: &str,
    timeout_rx: mpsc::Receiver<()>,
) -> AppResult<()> {
    loop {
        match timeout_rx.try_recv() {
            Ok(_) => {
                clean_up_v2(docker, container_id).await?;
                return Ok(());
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(e) => return Err(AppError::System(format!("mpsc channel error: {e}"))),
        }
    }
}

/// Removes a container forcefully.
///
/// # Arguments
///
/// * `docker` - Reference to the Docker client.
/// * `container_id` - ID of the container to remove.
async fn clean_up_v2(docker: &Docker, container_id: &str) -> AppResult<()> {
    docker
        .remove_container(
            container_id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| AppError::System(format!("Failed to remove container: {e}")))?;
    Ok(())
}

/// Calculates the CPU period and CPU quota for a given `x` (number of CPUs).
///
/// # Arguments
///
/// * `x` - The number of CPUs to allocate. For example, 1.0 = 1 CPU core,
///         2.0 = 2 CPU cores, 0.5 = half a CPU core, etc.
///
/// # Returns
///
/// A tuple `(cpu_period, cpu_quota)` suitable for use in Docker’s HostConfig.
fn cpu_limits(x: f64) -> (i64, i64) {
    // Docker's default CPU period is 100,000 microseconds (100ms).
    let cpu_period = 100_000_u64;

    // Round the quota to the nearest whole number of microseconds.
    let cpu_quota = (cpu_period as f64 * x).round() as i64;

    (cpu_period as i64, cpu_quota)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_limits() {
        // 1 CPU -> 100,000 microseconds
        let (period, quota) = cpu_limits(1.0);
        assert_eq!(period, 100_000);
        assert_eq!(quota, 100_000);

        // 2 CPUs -> 200,000 microseconds
        let (period, quota) = cpu_limits(2.0);
        assert_eq!(period, 100_000);
        assert_eq!(quota, 200_000);

        // Half a CPU -> 50,000 microseconds
        let (period, quota) = cpu_limits(0.5);
        assert_eq!(period, 100_000);
        assert_eq!(quota, 50_000);
    }
}

#[tokio::test]
async fn test_runner() {
    // Make sure the "hello-world" image is available locally or can be pulled.
    let result = runner("test-runner", "8080:8080", None).await;
    assert!(result.is_ok(), "Container should start successfully.");
}
