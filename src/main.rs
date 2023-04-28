pub mod app;
pub mod auth;
pub mod db;
pub mod errors;
pub mod logging;
pub mod schemas;

use app::{run_app, AvatarClient, Settings};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = logging::get_tracing_subscriber("info", std::io::stdout);
    logging::init_tracing_subscriber(subscriber).unwrap_or_else(|e| tracing::error!(e));


    //database setup
    let config = Settings::get_configuration().unwrap();
    let app_state = config
        .create_app_state()
        .await
        .expect("can not establish conncetion");

    sqlx::migrate!("./migrations")
        .run(&app_state.connection)
        .await
        .expect("Can not run migrations");

    //creating new client for gravatar API
    let avatar = AvatarClient::new(config.avatar.base_url, config.avatar.default_img);

    let listener = TcpListener::bind(format!("{}:{}", config.app.host, config.app.port))?;

    println!("Server started at: 127.0.0.1:{}", config.app.port);

    run_app(listener, app_state, avatar)?.await
}
