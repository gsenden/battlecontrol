use crate::ports::LoggerDrivingPort;
use common::domain::Error;
use common::domain::ErrorTrait;

#[derive(Clone, Copy)]
pub struct TracingLoggerAdapter;

impl LoggerDrivingPort for TracingLoggerAdapter {
    fn log_error(&self, error: &Error) {
        tracing::error!("{:?}: {:?}", error.key(), error.params());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn implements_logger_driving_port() {
        fn assert_logger<T: LoggerDrivingPort>() {}
        assert_logger::<TracingLoggerAdapter>();
    }
}
