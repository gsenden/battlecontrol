include!(concat!(env!("OUT_DIR"), "/env_var.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_database_path_value_returns_env_var() {
        unsafe {
            std::env::set_var("MATTER_SERVER_DATABASE_PATH", "/data/battlecontrol.db");
        }
        assert_eq!(EnvVar::ServerDatabasePath.value(), "/data/battlecontrol.db");
        unsafe {
            std::env::remove_var("MATTER_SERVER_DATABASE_PATH");
        }
    }

    #[test]
    fn server_database_path_value_returns_default() {
        assert_eq!(EnvVar::ServerDatabasePath.value(), "battlecontrol.db");
    }

    #[test]
    fn env_var_server_database_path_exists() {
        let _ = EnvVar::ServerDatabasePath;
    }

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
