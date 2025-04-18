use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn timeout(timeout: Duration) -> (mpsc::Receiver<()>, Box<dyn FnOnce()>) {
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

