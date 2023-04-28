pub mod avatar;
pub mod configurations;
pub mod services;

pub use avatar::*;
pub use configurations::*;
pub use services::*;

use std::net::TcpListener;

use crate::auth::{login_user, logout_handler, refresh_auth, register_user};
use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};

pub fn run_app(
    listener: TcpListener,
    app_state: AppState,
    avatar_client: AvatarClient,
) -> std::io::Result<Server> {
    let data = web::Data::new(app_state);
    let avatar_client = web::Data::new(avatar_client);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(avatar_client.clone())
            .wrap(Logger::default())
            .service(health_check)
            .service(index)
            .service(post_student)
            .service(get_all_students)
            .service(get_student)
            .service(change_student)
            .service(get_avatar)
            .service(delete_student)
            .service(register_user)
            .service(login_user)
            .service(logout_handler)
            .service(refresh_auth)
    })
    .listen(listener)?
    .run();

    Ok(server)
}
