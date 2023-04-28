use fake::{Dummy, Faker, Fake};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use zero2prod::schemas::User;

use crate::{post_students_tests::send_post_request, start_app};
use fake::faker::internet::en::{Username,Password,SafeEmail};

#[derive(Debug, Serialize, Deserialize, Dummy)]
pub struct FakeRegisterUser {
    #[dummy(faker = "Username()")]
    pub username: String,
    #[dummy(faker = "SafeEmail()")]
    pub email: String,
    #[dummy(faker = "Password(8..14)")]
    pub password: String,
}



#[sqlx::test]
async fn register_user(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;

    let new_user: FakeRegisterUser = Faker.fake();
    let register_user_add = format!("{}/auth/register", address);
    let response = send_post_request(&new_user, register_user_add).await?;

    assert!(response.status().is_success());
    let res_data = response.json::<User>().await?;

    assert_eq!(res_data.username, new_user.username);
    assert_eq!(res_data.email, new_user.email);

    Ok(())
}

#[sqlx::test]
async fn user_log_in(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;

    let new_user: FakeRegisterUser = Faker.fake();
    let register_user_add = format!("{}/auth/register", address);
    let response = send_post_request(&new_user, register_user_add).await?;

    assert!(response.status().is_success());
    let res_data = response.json::<User>().await?;

    assert_eq!(res_data.username, new_user.username);
    assert_eq!(res_data.email, new_user.email);

    let login_data=json!({
        "email":new_user.email,
        "password":new_user.password
    });

    let login_user_add = format!("{}/auth/login", address);
    let response = send_post_request(&login_data, login_user_add).await?;

    assert!(response.status().is_success());


    Ok(())
}