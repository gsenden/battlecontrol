include!(concat!(env!("OUT_DIR"), "/resource.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_exists() {
        let _ = Resource::AuthUser;
    }

    #[test]
    fn path_returns_route() {
        assert_eq!(Resource::AuthUser.path(), "/auth/user");
    }
}
