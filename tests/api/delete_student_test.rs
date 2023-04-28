use fake::{Fake, Faker};
use sqlx::PgPool;
use zero2prod::schemas::FullStudent;

use crate::{
    post_students_tests::{send_post_request, FakeStudent},
    start_app,
};

#[sqlx::test]
async fn delete_student_check(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;

    //add new student
    let new_student: FakeStudent = Faker.fake();
    let post_student_address = format!("{}/students", address);
    let response = send_post_request(&new_student, post_student_address).await?;

    assert!(response.status().is_success());
    let res_data = response.json::<FullStudent>().await?;

    assert_eq!(res_data.full_name, new_student.full_name);
    assert_eq!(res_data.email, new_student.email);
    assert_eq!(res_data.age, new_student.age);
    assert_eq!(res_data.courses, new_student.courses);

    //delete added student
    let response = reqwest::Client::new()
        .delete(format!("{}/delete/{}", address, res_data.id))
        .send()
        .await?;

    assert!(response.status().is_success());

    let res_data = response.json::<String>().await?;
    assert!(res_data.contains("Deleted user"));
    Ok(())
}
