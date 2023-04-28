use crate::{
    errors::{Auth, Error, ErrorTypes},
    schemas::{LoginUser, RegisterUser, User},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sqlx::{PgPool, Row};
use tracing::{instrument, Instrument};

#[instrument(name = "Add new user", skip(connection), ret(Debug))]
pub async fn db_add_user(data: RegisterUser, connection: &PgPool) -> Result<User, Error> {
    let query_span =
        tracing::info_span!("Check if another users with provided email or username exist");
    let exists: bool =
        sqlx::query("select exists(select 1 from users where email = $1 or username=$2)")
            .bind(&data.email)
            .bind(&data.username)
            .fetch_one(connection)
            .instrument(query_span)
            .await
            .map_err(|e| Error {
                cause: Some(e.to_string()),
                message: Some("Query error".into()),
                error_type: ErrorTypes::DbError,
            })?
            .get(0);

    if exists {
        tracing::error!(
            "User with username: '{}' or email: '{}' already exist",
            data.username,
            data.email
        );
        return Err(Error {
            cause: None,
            message: Some("User with that username or email already exist".to_string()),
            error_type: ErrorTypes::Auth(Auth::Authorization),
        });
    }
    //generate password hash
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(data.password.as_bytes(), &salt)
        .map_err(|e| Error {
            cause: Some(e.to_string()),
            message: Some("Password hashing error".into()),
            error_type: ErrorTypes::DbError,
        })?
        .to_string();

    let query_span = tracing::info_span!("Inserting new user to db");
    let query_result = sqlx::query_as!(
        User,
        "INSERT INTO users (username,email,password_hash) VALUES ($1, $2, $3) RETURNING *",
        data.username,
        data.email,
        hashed_password
    )
    .fetch_one(connection)
    .instrument(query_span)
    .await
    .map_err(|e| {
        Error::new(
            Some(e.to_string()),
            Some("Can not insert new user".into()),
            ErrorTypes::DbError,
        )
    })?;

    Ok(query_result)
}

#[instrument(name = "Find the user in db", skip(connection), ret(Debug))]
pub async fn db_find_user(user_id: uuid::Uuid, connection: &PgPool) -> Result<User, Error> {
    let query_span = tracing::info_span!("Query user",%user_id);
    let user = sqlx::query_as!(User, "select * from users where id=$1;", user_id)
        .fetch_one(connection)
        .instrument(query_span)
        .await
        .map_err(|e| {
            Error::new(
                Some(e.to_string()),
                Some("User doesn't exist".into()),
                ErrorTypes::NotFoundError,
            )
        })?;

    Ok(user)
}

#[instrument(name = "User login", skip(connection), ret(Debug))]
pub async fn user_login(data: LoginUser, connection: &PgPool) -> Result<User, Error> {
    let query_span = tracing::info_span!("Find user with the email");
    let user = sqlx::query_as!(User, "select * from users where email = $1", data.email)
        .fetch_one(connection)
        .instrument(query_span)
        .await
        .map_err(|e| Error {
            cause: Some(e.to_string()),
            message: Some("Invalid password or email".into()),
            error_type: ErrorTypes::Auth(Auth::Authentication),
        })?;

    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| Error {
        cause: Some(e.to_string()),
        message: Some("Password hashing error".into()),
        error_type: ErrorTypes::DbError,
    })?;

    Argon2::default()
        .verify_password(data.password.as_bytes(), &parsed_hash)
        .map_err(|e| Error {
            cause: Some(e.to_string()),
            message: Some("Incorrect password. Try again!".into()),
            error_type: ErrorTypes::Auth(Auth::Authentication),
        })?;

    Ok(user)
}
