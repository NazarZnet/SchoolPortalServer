use fake::faker::{internet::en::SafeEmail, lorem::en::Words, name::en::Name};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use zero2prod::schemas::FullStudent;

use crate::start_app;

use fake::{Dummy, Fake, Faker};
#[derive(Debug, Serialize, Deserialize, Dummy)]
pub struct FakeStudent {
    pub id:uuid::Uuid,
    #[serde(rename = "fullName")]
    #[dummy(faker = "Name()")]
    pub full_name: String,
    #[dummy(faker = "SafeEmail()")]
    pub email: String,
    #[dummy(faker = "16..100")]
    pub age: i32,
    #[dummy(faker = "Words(1..3)")]
    pub courses: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Dummy)]
pub struct FakeEditStudent {
    #[dummy(faker = "SafeEmail()")]
    pub email: String,
    #[dummy(faker = "16..100")]
    pub age: i32,
    #[dummy(faker = "Words(1..3)")]
    pub courses: Vec<String>,
}

pub async fn send_post_request<T: Serialize>(
    new_user: &T,
    address: String,
) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();

    client.post(address).json(new_user).send().await
}

#[sqlx::test]
async fn post_student_check(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;

    let new_student: FakeStudent = Faker.fake();
    let post_students_address = format!("{}/students", address);
    let response = send_post_request(&new_student, post_students_address).await?;

    assert!(response.status().is_success());
    let res_data = response.json::<FullStudent>().await?;

    assert_eq!(res_data.full_name, new_student.full_name);
    assert_eq!(res_data.email, new_student.email);
    assert_eq!(res_data.age, new_student.age);
    assert_eq!(res_data.courses, new_student.courses);

    Ok(())
}

#[should_panic]
#[sqlx::test]
async fn post_student_check_panic_validation(pool: PgPool) {
    let address = start_app(pool).await;

    //invalid email, age and course
    let new_student: FakeStudent = FakeStudent {
        id:uuid::Uuid::new_v4(),
        full_name: "Name Surname".to_owned(),
        email: "invalidemail".to_owned(),
        age: 14,
        courses: vec!["123course".to_owned()],
    };
    let post_student_address = format!("{}/students", address);
    let response = send_post_request(&new_student, post_student_address)
        .await
        .expect("Send request error");

    assert!(response.status().is_success());

    let res_data = response
        .json::<FullStudent>()
        .await
        .expect("Deserialization error");

    assert_eq!(res_data.full_name, new_student.full_name);
    assert_eq!(res_data.email, new_student.email);
    assert_eq!(res_data.age, new_student.age);
    assert_eq!(res_data.courses, new_student.courses);
}

#[sqlx::test]
async fn change_student(pool: PgPool) -> Result<(), reqwest::Error> {
    let address = start_app(pool).await;

    let new_student: FakeStudent = Faker.fake();
    let post_student_address = format!("{}/students", &address);
    let response = send_post_request(&new_student, post_student_address).await?;

    assert!(response.status().is_success());
    let res_data = response.json::<FullStudent>().await?;

    assert_eq!(res_data.full_name, new_student.full_name);
    assert_eq!(res_data.email, new_student.email);
    assert_eq!(res_data.age, new_student.age);
    assert_eq!(res_data.courses, new_student.courses);

    let new_student_data: FakeEditStudent = Faker.fake();

    let change_student_address = format!("{}/students/change/{}", &address, res_data.id);
    let response = send_post_request(&new_student_data, change_student_address).await?;

    assert!(response.status().is_success());

    let changed_user = response.json::<FakeStudent>().await?;

    assert_eq!(res_data.full_name, changed_user.full_name);
    assert_ne!(res_data.email, changed_user.email);
    assert_ne!(res_data.age, changed_user.age);
    assert_ne!(res_data.courses, changed_user.courses);

    Ok(())
}
