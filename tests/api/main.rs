pub mod avatar_tests;
pub mod delete_student_test;
pub mod get_students_tests;
pub mod health_check;
pub mod post_students_tests;
pub mod auth_user_tests;

use wiremock::{Match, Request};

use once_cell::sync::Lazy;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{
    schemas::FullStudent,
    logging::{get_tracing_subscriber, init_tracing_subscriber}, app::Settings,
};

use zero2prod::app::{run_app, AvatarClient};

static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let sub = get_tracing_subscriber("info", std::io::stdout);
        init_tracing_subscriber(sub).unwrap();
    } else {
        let sub = get_tracing_subscriber("info", std::io::sink);
        init_tracing_subscriber(sub).unwrap();
    }
});

struct HasEmailHash;

impl Match for HasEmailHash {
    fn matches(&self, request: &Request) -> bool {
        if let Some(hash) = request
            .url
            .to_string()
            .split("/")
            .nth(3)
            .unwrap()
            .split('?')
            .next()
        {
            return hash.len() == 32;
        }
        false
    }
}

async fn mock_avatar_client() -> AvatarClient {
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    //create mock server

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(HasEmailHash {})
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "/www.cornwallbusinessawards.co.uk/wp-content/uploads/2019/01/Person-icon.jpg",
        ))
        .mount(&mock_server)
        .await;

    AvatarClient::new(
        mock_server.uri(),
        "https://www.cornwallbusinessawards.co.uk/wp-content/uploads/2019/01/Person-icon.jpg"
            .into(),
    )
}

pub async fn start_app(pool: PgPool) -> String {
    Lazy::force(&TRACING);

    let settings=Settings::get_configuration().unwrap();
    let mut app_state=settings.create_app_state().await.unwrap();
    app_state.connection=pool;
    //run migrations for mock database
    sqlx::migrate!("./migrations")
        .run(&app_state.connection)
        .await
        .expect("Can not run migrations");

    
    let listener = TcpListener::bind("127.0.0.1:0").expect("Can not create address");

    let port = listener.local_addr().unwrap().port();

    println!("Server started at: 127.0.0.1:{}", port);
    let avatar = mock_avatar_client().await;

    let _s = tokio::spawn(run_app(listener, app_state, avatar).expect("Error bind server"));
    format!("http://127.0.0.1:{}", port)
}
