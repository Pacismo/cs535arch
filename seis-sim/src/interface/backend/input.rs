use std::{
    io::stdin,
    sync::mpsc::{channel, Receiver, RecvTimeoutError},
    thread::{spawn, JoinHandle},
    time::Duration,
};

pub struct InputHandler {
    _thread: JoinHandle<()>,
    recv: Receiver<String>,
}

impl InputHandler {
    pub fn new() -> Self {
        let (tx, recv) = channel();

        Self {
            _thread: spawn(move || loop {
                let mut input = String::new();
                stdin().read_line(&mut input).expect("Failed to read input");
                tx.send(input).expect("Failed to send message");
            }),
            recv,
        }
    }

    pub fn get_next_timeout(&mut self, timeout: Duration) -> Option<String> {
        match self.recv.recv_timeout(timeout) {
            Ok(cmd) => Some(cmd),
            Err(RecvTimeoutError::Timeout) => None,
            Err(RecvTimeoutError::Disconnected) => panic!("Input-handling thread terminated"),
        }
    }

    pub fn get_next(&mut self) -> String {
        match self.recv.recv() {
            Ok(cmd) => cmd,
            Err(e) => panic!("{e}"),
        }
    }
}
