use common::domain::Error;

pub trait LoggerDrivingPort: Send + Sync + 'static {
    fn log_error(&self, error: &Error);
}
