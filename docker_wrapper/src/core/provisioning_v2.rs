use std::fs::File;
use std::io::Write;

use bollard::Docker;
use bollard::errors::Error as BollardError;
use bollard::image::BuildImageOptions;
use futures_util::StreamExt;
use tar::Builder as TarBuilder;
use tempfile::TempDir;

use crate::shared::error::{AppResult, Error as AppError};
use crate::shared::utils::print_output; // your logging/printing utility

/// Creates a tar archive (in a temp directory) containing the provided Dockerfile content.
/// Returns a `Body` that can be streamed to the Docker daemon.
///
/// # Arguments
/// * `dockerfile_content` - The Dockerfile content in a string.
///
/// # Returns
/// * On success, returns `(Body, TempDir)` where `Body` is the tar'd build context,
///   and `TempDir` is the reference to the created temp dir (so it doesn't go out of scope).
/// * On failure, returns an `AppError`.
fn create_build_context(dockerfile_content: &str) -> AppResult<(Vec<u8>, TempDir)> {
    // 1. Create a temporary directory to hold the Dockerfile.
    let temp_dir = tempfile::tempdir()
        .map_err(|e| AppError::System(format!("Failed to create temp dir: {e}")))?;

    // 2. Write the Dockerfile content into that directory.
    let dockerfile_path = temp_dir.path().join("Dockerfile");
    {
        let mut file = File::create(&dockerfile_path)
            .map_err(|e| AppError::System(format!("Failed to create Dockerfile: {e}")))?;
        file.write_all(dockerfile_content.as_bytes())
            .map_err(|e| AppError::System(format!("Failed to write Dockerfile: {e}")))?;
    }

    // 3. Create a tar archive containing the Dockerfile.
    let tar_path = temp_dir.path().join("context.tar");
    {
        let tar_file = File::create(&tar_path)
            .map_err(|e| AppError::System(format!("Failed to create tar: {e}")))?;
        let mut tar_builder = TarBuilder::new(tar_file);
        tar_builder
            .append_path_with_name(&dockerfile_path, "Dockerfile")
            .map_err(|e| AppError::System(format!("Failed to append file to tar: {e}")))?;
    }

    // 4. Read the tar file into memory so it can be streamed.
    let tar_data = std::fs::read(&tar_path)
        .map_err(|e| AppError::System(format!("Failed to read tar file: {e}")))?;

    // Return the tar archive as a `Body` plus the `TempDir` to keep it alive.
    Ok((tar_data, temp_dir))
}

/// Builds a Docker image from the given Dockerfile content using Bollard.
///
/// # Arguments
/// * `runner_type`        - The Docker image name/tag (e.g., "python-runner").
/// * `dockerfile_content` - The Dockerfile contents as a string.
///
/// # Returns
/// * `Ok(())` if the image build succeeds.
/// * `AppError` if there's a problem connecting to Docker or building the image.
pub async fn provisioning(runner_type: &str, dockerfile_content: &str) -> AppResult<()> {
    // 1. Connect to Docker (verifies Docker is installed and running).
    let docker = Docker::connect_with_socket_defaults()
        .map_err(|e| AppError::System(format!("Unable to connect to Docker: {e}")))?;

    // 2. Create the build context as a tar archive (in memory).
    let (build_context, _temp_dir) = create_build_context(dockerfile_content)?;

    // 3. Set the build options (like `docker build -t runner_type .`).
    let build_options = BuildImageOptions {
        t: runner_type,
        rm: true, // remove intermediate containers on success
        ..Default::default()
    };

    // 4. Call Bollard's build_image, streaming the archive.
    let mut build_stream = docker.build_image(build_options, None, Some(build_context.into()));

    // 5. Process the build output stream.
    while let Some(build_info_result) = build_stream.next().await {
        match build_info_result {
            Ok(build_info) => {
                // Bollard returns JSON about each build step.
                println!("Status: {:?}", build_info.status);
            }
            Err(BollardError::DockerResponseServerError { message, .. }) => {
                return Err(AppError::Exec(format!("Docker build error: {message}")));
            }
            Err(e) => {
                return Err(AppError::Exec(format!("Build stream error: {e}")));
            }
        }
    }

    println!("Environment provisioned (Docker image built successfully).");
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    // Use #[tokio::test] for async tests.
    #[tokio::test]
    async fn test_provisioning() {
        let dockerfile_content = r###"
            # Use an official Python runtime as a parent image
            FROM python:3.8

            # Set the working directory in the container
            WORKDIR /usr/src/app

            # Define the command to run when the container starts
            ENTRYPOINT ["python", "-c"]
        "###;

        let result = provisioning("test-runner", dockerfile_content).await;
        assert!(result.is_ok(), "Expected provisioning to succeed");

        // Give Docker a moment to finish any background cleanup
        sleep(Duration::from_secs(1)).await;
    }
}
