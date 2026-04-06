use std::collections::HashMap;

pub trait ErrorTrait {
    fn key(&self) -> super::ErrorCode;
    fn params(&self) -> &HashMap<String, String>;

    fn translate(&self, lang: super::Language) -> String {
        let i18n_key: super::I18n = self.key().into();
        let mut text = i18n_key.translate(lang).to_string();
        for (param, value) in self.params() {
            text = text.replace(&format!("{{{param}}}"), value);
        }
        text
    }
}

include!(concat!(env!("OUT_DIR"), "/error.rs"));

impl serde::Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Error", 2)?;
        state.serialize_field("code", &self.key())?;
        state.serialize_field("params", self.params())?;
        state.end()
    }
}

#[cfg(feature = "reqwest")]
impl Error {
    pub fn from_reqwest(error: reqwest::Error, url: String) -> Self {
        if error.is_connect() {
            Error::ServerOffline(ServerOfflineError::new(url))
        } else if error.is_timeout() {
            Error::RequestTimeout(RequestTimeoutError::new(url))
        } else {
            Error::RequestFailed(RequestFailedError::new(url))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Language;

    #[test]
    fn serialize_includes_error_code() {
        let error = Error::RoomNotFound(RoomNotFoundError::new("lobby".to_string()));
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"code\":\"RoomNotFound\""));
    }

    #[test]
    fn serialize_includes_params() {
        let error = Error::RoomNotFound(RoomNotFoundError::new("lobby".to_string()));
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"room_name\":\"lobby\""));
    }

    #[test]
    fn server_offline_error_exists() {
        let _ = Error::ServerOffline(ServerOfflineError::new("localhost".to_string()));
    }

    #[test]
    fn room_not_found_error_exists() {
        let _ = Error::RoomNotFound(RoomNotFoundError::new("lobby".to_string()));
    }

    #[test]
    fn translate_fills_params() {
        let error = Error::ServerOffline(ServerOfflineError::new("localhost".to_string()));
        let text = error.translate(Language::default());
        assert_eq!(text, "Verbinding met de server localhost is mislukt");
    }

    #[test]
    fn translate_works_for_other_language() {
        let error = Error::RoomNotFound(RoomNotFoundError::new("lobby".to_string()));
        let text = error.translate(Language::EnGb);
        assert_eq!(text, "Room lobby not found");
    }
}
