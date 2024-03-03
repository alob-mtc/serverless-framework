use std::process::Output;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn timeout(timeout: Duration) -> (mpsc::Receiver<()>, Box<dyn FnOnce() -> ()>) {
    let (tx, rx) = mpsc::channel();

    // The closure to trigger the timeout
    let tiger = Box::new(move || {
        thread::spawn(move || {
            thread::sleep(timeout);
            let _ = tx.send(());
        });
    });

    (rx, tiger)
}

pub fn print_output(output: &Output) {
    // Print the output of the command
    println!("Status: {}", output.status);
    // Convert the output to a String and print it
    println!("Stdout: \n{}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
}
