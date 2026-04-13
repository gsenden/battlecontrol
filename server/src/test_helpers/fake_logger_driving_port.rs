use crate::ports::LoggerDrivingPort;
use common::domain::Error;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct FakeLoggerDrivingPort {
    logged_errors: Arc<Mutex<Vec<Error>>>,
}

impl FakeLoggerDrivingPort {
    pub fn new() -> Self {
        Self {
            logged_errors: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn logged_errors(&self) -> Vec<Error> {
        self.logged_errors.lock().unwrap().clone()
    }
}

impl LoggerDrivingPort for FakeLoggerDrivingPort {
    fn log_error(&self, error: &Error) {
        self.logged_errors.lock().unwrap().push(error.clone());
    }
}
