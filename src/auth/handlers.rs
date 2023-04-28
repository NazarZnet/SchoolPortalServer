use actix_web::{
    cookie::{time::Duration as ActixWebDuration, Cookie},
    get, post, web, HttpRequest, HttpResponse, Responder, ResponseError,
};

use serde_json::json;
use time::Duration;
use tracing::instrument;
use validator::Validate;

use crate::{
    app::AppState,
    auth::JwtMiddleware,
    db::{db_add_user, db_find_user, user_login},
    errors::{Error, ErrorTypes},
    schemas::{LoginUser, RegisterUser, TokenClaims, TokenType},
};

#[post("/auth/register")]
#[instrument(skip(state), name = "Register user")]
pub async fn register_user(
    data: web::Json<RegisterUser>,
    state: web::Data<AppState>,
) -> impl Responder {
    if let Err(error) = data.validate().map_err(|e| {
        Error::new(
            Some(serde_json::to_string_pretty(&e).unwrap()),
            Some("Invalid data".into()),
            ErrorTypes::ValidationError,
        )
    }) {
        tracing::error!("Invalid input data. Errors: {}", error);
        return error.error_response();
    }

    match db_add_user(data.into_inner(), &state.connection).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            tracing::error!("Error insert new user: '{:?}'", e);
            e.error_response()
        }
    }
}
#[post("/auth/login")]
#[instrument(skip(state), name = "User login")]
async fn login_user(data: web::Json<LoginUser>, state: web::Data<AppState>) -> impl Responder {
    if let Err(error) = data.validate().map_err(|e| {
        Error::new(
            Some(serde_json::to_string_pretty(&e).unwrap()),
            Some("Invalid data".into()),
            ErrorTypes::ValidationError,
        )
    }) {
        tracing::error!("Invalid input data. Errors: {}", error);
        return error.error_response();
    }

    let user = match user_login(data.into_inner(), &state.connection).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Error insert new user: '{:?}'", e);
            return e.error_response();
        }
    };
    let access_token = match state.jwt.encode(
        &TokenClaims::new(user.id.to_string(), state.jwt.access.exp),
        TokenType::Access,
    ) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Error creating new access token");
            return e.error_response();
        }
    };

    let refresh_token = match state.jwt.encode(
        &TokenClaims::new(user.id.to_string(), state.jwt.refresh.exp),
        TokenType::Refresh,
    ) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Error creating new refresh token");
            return e.error_response();
        }
    };

    let cookie = Cookie::build("access_token", access_token.to_owned())
        .path("/")
        .max_age(state.jwt.access.maxage)
        .http_only(true)
        .finish();

    let cookie2 = Cookie::build("refresh_token", refresh_token.to_owned())
        .path("/")
        .max_age(state.jwt.refresh.maxage)
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .cookie(cookie2)
        .json(json!({"status": "success", "access": access_token,"refresh":refresh_token}))
}

#[get("/auth/refresh")]
#[instrument(skip_all, name = "User refresh authorization")]
async fn refresh_auth(
    req: HttpRequest,
    state: web::Data<AppState>,
    _: JwtMiddleware,
) -> impl Responder {
    //check refresh token and find User's id
    let user_id = match state.jwt.refresh(&req) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Can not find user's id");
            return e.error_response();
        }
    };

    //check if logged use exist and Uuid valid
    let user = match db_find_user(user_id, &state.connection).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Can not find user with id '{}'", user_id);
            return e.error_response();
        }
    };

    let new_token = match state.jwt.encode(
        &TokenClaims::new(user.id.to_string(), Duration::minutes(1)),
        TokenType::Access,
    ) {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Error creating new access token");
            return e.error_response();
        }
    };

    let cookie = Cookie::build("access_token", new_token.to_owned())
        .path("/")
        .max_age(ActixWebDuration::new(60 * 60, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(json!({"status": "success", "new_access": new_token}))
}

#[get("/auth/logout")]
#[instrument(name = "User logout")]
async fn logout_handler(_: JwtMiddleware) -> impl Responder {
    let cookie = Cookie::build("refresh_token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();

    let cookie2 = Cookie::build("access_token", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .cookie(cookie2)
        .json(json!({"status": "success"}))
}
