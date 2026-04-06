pub trait ApiAdapter {
    fn routes(self) -> axum::Router;
}
