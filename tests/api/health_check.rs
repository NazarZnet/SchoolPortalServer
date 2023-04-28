use crate::start_app;
use sqlx::PgPool;

#[sqlx::test]
async fn server_health_check(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;

    let response = reqwest::get(format!("{}/health_check", address)).await?;

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    Ok(())
}

#[sqlx::test]
async fn index_route_test(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;

    let response = reqwest::get(format!("{}/health_check", address)).await?;
    assert!(response.status().is_success());
    assert_ne!(response.headers().get("content-length").unwrap(), &"10");

    Ok(())
}
