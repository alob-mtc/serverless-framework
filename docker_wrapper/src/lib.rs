mod error;
mod utils;

use error::{AppResult, Error as AppError};
use std::fs::File;
use std::io::Write;
use std::{
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use utils::{print_output, timeout};
use uuid::Uuid;

static PROGRAM: &str = "docker";
static TIMEOUT: u64 = 3;

pub fn runner(runner_type: &str, port: &str, timeout_val: u64) -> Option<String> {
    let start = Instant::now();
    let my_uuid = Uuid::new_v4().to_string();
    let mut child = Command::new(PROGRAM)
        .arg("run")
        .arg("--name")
        .arg(&my_uuid)
        .args(["-m", "256m"])
        .args(["--cpus", "2.0"])
        .arg("-p")
        .arg(port)
        .arg(runner_type)
        .spawn()
        .expect(r###"Failed to execute "run" command"###);

    let container_id = my_uuid.clone();
    thread::spawn(move || {
        let timeout_val = if timeout_val > 0 {
            timeout_val
        } else {
            TIMEOUT
        };

        let (rx, trigger_timeout) = timeout(Duration::from_secs(timeout_val));
        // Trigger the timeout mechanism
        trigger_timeout();

        match monitor_child_process(&mut child, rx) {
            Ok(_) => {
                let elapsed_time = start.elapsed();
                println!(
                    "execution took {} seconds.",
                    (elapsed_time.as_millis() as f64 / 1000.0)
                );
                let output = child.wait_with_output().unwrap();
                print_output(&output);
            }
            Err(e) => eprintln!("{e}"),
        }

        // clean up
        if let Err(e) = clean_up(&container_id) {
            eprintln!("Failed to execute clean up command: {e}");
        }
    });

    Some(my_uuid)
}

fn monitor_child_process(child: &mut Child, timeout_rx: mpsc::Receiver<()>) -> AppResult<()> {
    loop {
        match timeout_rx.try_recv() {
            Ok(_) => {
                try_wait(child, true)?;
                return Ok(());
            }
            Err(mpsc::TryRecvError::Empty) => {
                let completed = try_wait(child, false)?;
                if completed {
                    return Ok(());
                }
            }
            Err(e) => {
                return Err(AppError::System(format!("{e}")));
            }
        }
    }
}

fn try_wait(child: &mut Child, timeout_killer: bool) -> AppResult<bool> {
    match child
        .try_wait()
        .map_err(|e| -> AppError { AppError::System(format!("Error attempting to wait: {e}")) })?
    {
        Some(status) => {
            if !status.success() {
                return Err(AppError::Exec(format!(
                    "Process exits with status: {status}"
                )));
            }
            Ok(true)
        }
        None => {
            // if child process not done, and it's a timeout, kill the child process
            if timeout_killer {
                child.kill().map_err(|e| -> AppError {
                    AppError::System(format!("Error while trying to kill child process {e}"))
                })?;
                return Err(AppError::Exec("Process timed out".to_string()));
            }
            Ok(false)
        }
    }
}

fn clean_up(container_name: &str) -> AppResult<()> {
    Command::new(PROGRAM)
        .args(["rm", "-f"])
        .arg(container_name)
        .output()
        .map_or_else(
            |e| Err(AppError::Exec(format!("Failed to execute command: {}", e))),
            |output| {
                print_output(&output);
                Ok(())
            },
        )
}

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
pub fn provisioning_v2(runner_type: &str, dockerfile_content: &str) -> AppResult<()> {
    // check it docker is installed
    match Command::new("which").arg(&PROGRAM).output() {
        Ok(output) => {
            if !output.status.success() {
                return Err(AppError::Exec(
                    "Docker not installed on host machine".to_string(),
                ));
            }
            // build the image
            match Command::new(PROGRAM)
                .arg("build")
                .arg("-t")
                .arg(runner_type)
                .arg("-")
                .stdin(Stdio::piped())
                .spawn()
            {
                Ok(mut child) => match child.stdin.as_mut() {
                    None => {
                        todo!()
                    }
                    Some(stdin) => {
                        stdin
                            .write_all(dockerfile_content.as_bytes())
                            .map_err(|e| AppError::System(e.to_string()))?;
                        let output = child
                            .wait_with_output()
                            .map_err(|e| AppError::System(e.to_string()))?;
                        if !output.status.success() {
                            print_output(&output);
                            return Err(AppError::Exec("Failed to build docker image".to_string()));
                        }
                        println!("ENV provisioned");
                        Ok(())
                    }
                },
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
    fn test_runner() {
        // the provided image should be available
        runner("python-runner", "8080:8080", 3);
    }

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
