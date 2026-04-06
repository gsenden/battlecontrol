mod adapters;
mod ports;
mod domain;

#[cfg(test)]
mod test_helpers;

use adapters::{AuthApiAdapter, AxumAdapter, SqliteUserRepository, TracingLoggerAdapter};
use adapters::db::SqliteAdapter;
use domain::{Authenticator, AuthenticatorDrivenPorts};

struct ProductionDrivenPorts;
impl AuthenticatorDrivenPorts for ProductionDrivenPorts {
    type UserRepo = SqliteUserRepository;
}

#[tokio::main]
async fn main() {
    let sqlite = SqliteAdapter::new("battlecontrol.db")
        .expect("Failed to open database");
    let user_repo = SqliteUserRepository::new(sqlite)
        .expect("Failed to initialize user repository");
    let authenticator = Authenticator::<ProductionDrivenPorts>::new(user_repo);
    let logger = TracingLoggerAdapter;

    AxumAdapter::new()
        .register(AuthApiAdapter::new(authenticator, logger))
        .serve_spa("frontend/build")
        .serve()
        .await;
}
