include!(concat!(env!("OUT_DIR"), "/env_var.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_var_exists() {
        let _ = EnvVar::ServerHost;
    }

    #[test]
    fn value_returns_default() {
        assert_eq!(EnvVar::ServerPort.value(), "3000");
    }

    #[test]
    fn value_returns_env_var() {
        unsafe {
            std::env::set_var("MATTER_SERVER_HOST", "example.com");
        }
        assert_eq!(EnvVar::ServerHost.value(), "example.com");
        unsafe {
            std::env::remove_var("MATTER_SERVER_HOST");
        }
    }
}
