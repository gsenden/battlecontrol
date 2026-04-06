include!(concat!(env!("OUT_DIR"), "/language_generated.rs"));

#[cfg(test)]
mod tests {
    use crate::domain::Language;

    #[test]
    fn display_name_returns_short_code() {
        assert_eq!(Language::NlNl.display_name(), "NL");
    }

    #[test]
    fn available_language_exists() {
        let _ = Language::default();
    }
}
