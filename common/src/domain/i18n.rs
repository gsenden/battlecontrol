include!(concat!(env!("OUT_DIR"), "/i18n_generated.rs"));

#[cfg(test)]
mod tests {
    use crate::domain::{I18n, Language};

    #[test]
    fn translate_returns_text_for_en_gb() {
        assert_eq!(
            I18n::RoomNotFound.translate(Language::EnGb),
            "Room {room_name} not found"
        );
    }

    #[test]
    fn translate_returns_text_for_default_language() {
        assert_eq!(
            I18n::RoomNotFound.translate(Language::default()),
            "Kamer {room_name} niet gevonden"
        );
    }

    #[test]
    fn i18n_key_exists() {
        let _ = I18n::RoomNotFound;
    }
}
