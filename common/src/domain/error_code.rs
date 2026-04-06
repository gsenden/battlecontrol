include!(concat!(env!("OUT_DIR"), "/error_code_generated.rs"));

#[cfg(test)]
mod tests {
    use crate::domain::{ErrorCode, I18n};

    #[test]
    fn error_code_converts_to_i18n() {
        let _i18n: I18n = ErrorCode::RoomNotFound.into();
    }

    #[test]
    fn error_code_exists() {
        let _ = ErrorCode::RoomNotFound;
    }
}
