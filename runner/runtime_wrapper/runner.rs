use super::*;

use error::{AppResult, Error as AppError};
use std::{
    process::{Child, Command},
    sync::mpsc,
    time::{Duration, Instant},
};
use utils::{print_output, timeout};
use uuid::Uuid;

static RUNNER: &str = "code-runner";
static PROGRAM: &str = "docker";
static TIMEOUT: u64 = 3;

pub fn runner(runner_type: &str, code_snippet: &str, timeout_val: u64) {
    let runner_type = if !runner_type.is_empty() {
        runner_type
    } else {
        RUNNER
    };
    // provision env
    if let Err(e) = provisioning(runner_type) {
        eprintln!("Failed to provision code-runner env: {e}");
        return;
    }

    let start = Instant::now();
    let my_uuid = Uuid::new_v4().to_string();
    let mut child = Command::new(PROGRAM)
        .arg("run")
        .arg("--name")
        .arg(&my_uuid)
        .args(["-m", "256m"])
        .args(["--cpus", "2.0"])
        .arg(runner_type)
        .arg(code_snippet)
        .spawn()
        .expect(r###"Failed to execute "run" command"###);

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
    if let Err(e) = clean_up(my_uuid) {
        eprintln!("Failed to execute clean up command: {e}");
    }
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
            return Ok(true);
        }
        None => {
            // if child process not don and it's a timeout, kill the child process
            if timeout_killer {
                child.kill().map_err(|e| -> AppError {
                    AppError::System(format!("Error while trying to kill child process {e}"))
                })?;
                return Err(AppError::Exec("Process timed out".to_string()));
            }
            return Ok(false);
        }
    }
}

fn clean_up(container_name: String) -> AppResult<()> {
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

fn provisioning(runner_type: &str) -> AppResult<()> {
    // check it docker is installed
    match Command::new("which").arg(&PROGRAM).output() {
        Ok(output) => {
            if !output.status.success() {
                return Err(AppError::Exec(
                    "Docker not installed on host marchine".to_string(),
                ));
            }
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
                        print_output(&output)
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
