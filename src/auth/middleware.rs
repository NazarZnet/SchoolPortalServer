use actix_web::{dev::Payload, web, FromRequest, HttpMessage, HttpRequest};

use time::OffsetDateTime;
use tracing::instrument;

use std::future::{ready, Ready};

use crate::{
    app::AppState,
    errors::{Auth, Error, ErrorTypes},
    schemas::TokenType,
};

//custom middleware to check if token exist in request and refresh it
#[derive(Debug)]
pub struct JwtMiddleware {
    pub user_id: uuid::Uuid,
}

impl FromRequest for JwtMiddleware {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    #[instrument(skip_all,name="Check authorization",fields(uri = %req.uri(), method=%req.method()))]
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let state = req
            .app_data::<web::Data<AppState>>()
            .expect("Can not get app state data");

        tracing::info!("Get access jwt token from cookies");
        let tokens = match req.cookie("access_token").map(|c| c.value().to_string()) {
            Some(token) => token,
            None => {
                tracing::error!("Access token not found");
                return ready(Err(Error {
                    cause: None,
                    message: Some("Access token not found. Log in first!".into()),
                    error_type: ErrorTypes::Auth(Auth::Authorization),
                }));
            }
        };

        let token = match state.jwt.decode(&tokens, TokenType::Access) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Invalid jwt access token");

                return ready(Err(Error {
                    cause: Some(e.to_string()),
                    message: Some("Invalid jwt access token".into()),
                    error_type: ErrorTypes::JwtError,
                }));
            }
        };

        //check if token is valid only if it's not a refresh request
        if req.uri() != "/auth/refresh" {
            if OffsetDateTime::now_utc().unix_timestamp() as usize > token.exp {
                tracing::error!("Log in timed out");

                return ready(Err(Error {
                    cause: None,
                    message: Some("Login timed out".into()),
                    error_type: ErrorTypes::Auth(Auth::Authorization),
                }));
            }
        }

        //insert Uuid to request
        let user_id = uuid::Uuid::parse_str(token.sub.as_str()).unwrap();
        req.extensions_mut()
            .insert::<uuid::Uuid>(user_id.to_owned());

        ready(Ok(JwtMiddleware { user_id }))
    }
}
