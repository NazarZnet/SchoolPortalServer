use crate::post_students_tests::{send_post_request, FakeStudent};
use crate::{mock_avatar_client, start_app};
use sqlx::PgPool;

use fake::faker::internet::en::SafeEmail;
use fake::{Fake, Faker};
use zero2prod::schemas::FullStudent;

#[tokio::test]
async fn avatar_mock_api_test() -> Result<(), reqwest::Error> {
    let avatar_client = mock_avatar_client().await;
    let img = avatar_client
        .send_request(&SafeEmail().fake::<String>())
        .await?;
    assert!(img.len() > 0);
    assert!(img.contains(&format!("{}", avatar_client.base_url)));

    Ok(())
}

#[sqlx::test]
async fn student_avatar_check(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;

    let new_student: FakeStudent = Faker.fake();
    let post_student_address = format!("{}/students", address);
    let response = send_post_request(&new_student, post_student_address).await?;

    assert!(response.status().is_success());

    let user = response.json::<FullStudent>().await?;

    let avatar_response = reqwest::get(format!("{}/students/{}/avatar", address, user.id)).await?;

    assert!(avatar_response.status().is_success());
    let avatar_data = avatar_response.json::<String>().await?;
    assert!(avatar_data.len() > 0);

    Ok(())
}
