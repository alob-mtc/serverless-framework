use crate::shared::error::{AppResult, Error as AppError};
use bollard::errors::Error as BollardError;
use bollard::image::BuildImageOptions;
use bollard::Docker;
use fn_utils;
use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tar::Builder as TarBuilder;

/// Creates a tar archive (in a temp directory) containing the provided Dockerfile content.
/// Returns a `Body` that can be streamed to the Docker daemon.
///
/// # Arguments
/// * `dockerfile_content` - The Dockerfile content in a string.
///
/// # Returns
/// * On success, returns `Body` where `Body` is the tar'd build context,
/// * On failure, returns an `AppError`.
fn create_build_context(path: &PathBuf, dockerfile_content: &str) -> AppResult<Vec<u8>> {
    // Write the Dockerfile content into that directory.
    let dockerfile_path = path.join("Dockerfile");
    {
        let mut file = File::create(&dockerfile_path)
            .map_err(|e| AppError::System(format!("Failed to create Dockerfile: {e}")))?;
        file.write_all(dockerfile_content.as_bytes())
            .map_err(|e| AppError::System(format!("Failed to write Dockerfile: {e}")))?;
    }

    // Create a tar archive and copy over the content of path/<function_name>.
    // Including the Dockerfile.
    let tar_path = path.join("context.tar");
    {
        let tar_file = File::create(&tar_path)
            .map_err(|e| AppError::System(format!("Failed to create tar: {e}")))?;
        let mut tar_builder = TarBuilder::new(tar_file);
        fn_utils::add_dir_to_tar(&mut tar_builder, path, path, &[])
            .map_err(|e| AppError::System(format!("Failed to write Dockerfile: {e}")))?;
    }

    // Read the tar file into memory so it can be streamed.
    let tar_data = std::fs::read(&tar_path)
        .map_err(|e| AppError::System(format!("Failed to read tar file: {e}")))?;

    Ok(tar_data)
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
pub async fn provisioning(
    path: &PathBuf,
    runner_type: &str,
    dockerfile_content: &str,
) -> AppResult<()> {
    let docker = Docker::connect_with_socket_defaults()
        .map_err(|e| AppError::System(format!("Unable to connect to Docker: {e}")))?;

    // Create the build context as a tar archive (in memory).
    let build_context = create_build_context(path, dockerfile_content)?;

    let build_options = BuildImageOptions {
        t: runner_type,
        rm: true, // remove intermediate containers on success
        ..Default::default()
    };

    let mut build_stream = docker.build_image(build_options, None, Some(build_context.into()));

    // Process the build output stream.
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

        let temp_dir = tempfile::tempdir().unwrap().into_path();
        let result = provisioning(&temp_dir, "test-runner", dockerfile_content).await;
        assert!(result.is_ok(), "Expected provisioning to succeed");
    }
}
