use crate::shared::error::{AppResult, Error as AppError};
use crate::shared::utils::print_output;
use std::fs::File;
use std::io::Write;
use std::process::Command;

static PROGRAM: &str = "docker";

pub fn provisioning(runner_type: &str, dockerfile_content: &str) -> AppResult<()> {
    // check it docker is installed
    match Command::new("which").arg(&PROGRAM).output() {
        Ok(output) => {
            if !output.status.success() {
                return Err(AppError::Exec(
                    "Docker not installed on host machine".to_string(),
                ));
            }

            // create docker file
            File::create("Dockerfile")
                .map_err(|e| AppError::System(e.to_string()))?
                .write_all(dockerfile_content.as_bytes())
                .map_err(|e| AppError::System(e.to_string()))?;

            // build the image
            // docker build -t python-runner .
            match Command::new(PROGRAM)
                .arg("build")
                .arg("-t")
                .arg(runner_type)
                .arg(".")
                .output()
            {
                Ok(output) => {
                    if !output.status.success() {
                        print_output(&output);
                        return Err(AppError::Exec("Failed to build docker image".to_string()));
                    }
                    println!("ENV provisioned");
                    Ok(())
                }
                Err(e) => Err(AppError::System(e.to_string())),
            }
        }
        Err(e) => Err(AppError::System(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_provisioning() {
        let dockerfile_content = r###"
            # Use an official Python runtime as a parent image
            FROM python:3.8

            # Set the working directory in the container
            WORKDIR /usr/src/app

            # When running the container, Python will be invoked
            ENTRYPOINT ["python", "-c"]
        "###;

        let res = provisioning("test-runner", dockerfile_content);
        assert!(res.is_ok());
    }
}
