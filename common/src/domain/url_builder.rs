use super::Error;
use super::error::InvalidUrlError;

pub struct UrlBuilder;

impl UrlBuilder {
    pub fn build(host: &str, port: &str, path: &str) -> Result<url::Url, Error> {
        let host = host.trim_end_matches(':');
        let path = path.trim_end_matches('/');
        let path = if path.starts_with('/') { path.to_string() } else { format!("/{path}") };

        let mut url = url::Url::parse("http://placeholder")
            .map_err(|_| Error::InvalidUrl(InvalidUrlError::new(
                host.to_string(), port.to_string(), path.to_string(),
            )))?;
        let make_err = || Error::InvalidUrl(InvalidUrlError::new(
            host.to_string(), port.to_string(), path.clone(),
        ));

        url.set_host(Some(host)).map_err(|_| make_err())?;
        url.set_port(Some(port.parse().map_err(|_| make_err())?))
            .map_err(|_| make_err())?;
        url.set_path(&path);
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_trailing_slash_from_path() {
        let url = UrlBuilder::build("localhost", "3000", "auth/user/").unwrap();
        assert_eq!(url.as_str(), "http://localhost:3000/auth/user");
    }

    #[test]
    fn adds_leading_slash_to_path() {
        let url = UrlBuilder::build("localhost", "3000", "auth/user").unwrap();
        assert_eq!(url.as_str(), "http://localhost:3000/auth/user");
    }

    #[test]
    fn strips_trailing_colon_from_host() {
        let url = UrlBuilder::build("localhost:", "3000", "/auth/user").unwrap();
        assert_eq!(url.as_str(), "http://localhost:3000/auth/user");
    }

    #[test]
    fn builds_url() {
        let url = UrlBuilder::build("localhost", "3000", "/auth/user").unwrap();
        assert_eq!(url.as_str(), "http://localhost:3000/auth/user");
    }
}
