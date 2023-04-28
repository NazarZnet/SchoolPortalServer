use crate::{
    post_students_tests::{send_post_request, FakeStudent},
    start_app,
};
use fake::{Fake, Faker};
use sqlx::PgPool;
use zero2prod::schemas::FullStudent;

#[sqlx::test]
async fn get_students_check(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;
    let response = reqwest::get(format!("{}/students", address)).await?;
    assert!(response.status().is_success());

    let res_data = response.json::<Vec<FullStudent>>().await?;
    assert!(res_data.is_empty());
    Ok(())
}

#[sqlx::test]
async fn get_student_check(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;

    let new_student: FakeStudent = Faker.fake();
    let post_student_address = format!("{}/users", address);
    let response = send_post_request(&new_student, post_student_address).await?;

    assert!(response.status().is_success());

    let response = reqwest::get(format!("{}/users/{}", address, new_student.id)).await?;
    assert!(response.status().is_success());

    let res_data = response.json::<FullStudent>().await?;
    assert_eq!(res_data.full_name, new_student.full_name);
    assert_eq!(res_data.email, new_student.email);
    assert_eq!(res_data.age, new_student.age);
    assert_eq!(res_data.courses, new_student.courses);
    Ok(())
}
