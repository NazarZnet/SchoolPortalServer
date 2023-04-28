use std::sync::Arc;

use crate::{
    app::AvatarClient,
    errors::{Error, ErrorTypes},
    schemas::{AddStudent, EditStudent, FullStudent, Student},
};
use sqlx::PgPool;
use time::OffsetDateTime;
use tracing::{instrument, Instrument};
use uuid::Uuid;

#[instrument(name = "Get students from db", skip(connection))]
pub async fn db_get_student(student_id: Uuid, connection: &PgPool) -> Result<FullStudent, Error> {
    let query_span = tracing::info_span!("Get user",%student_id);
    //get student from students table without courses
    let student = sqlx::query_as!(Student, "select * from students where id=$1", student_id)
        .fetch_one(connection)
        .instrument(query_span)
        .await
        .map_err(|e| {
            Error::new(
                Some(e.to_string()),
                Some("Can not find student with the provided id".into()),
                ErrorTypes::DbError,
            )
        })?;

    let courses = get_courses(student_id, connection).await?;

    Ok(FullStudent {
        id: student.id,
        full_name: student.full_name,
        email: student.email,
        age: student.age,
        img: student.img,
        registration_date: student.registration_date,
        courses,
    })
}

#[instrument(name = "Get all students from db", skip(connection), ret(Debug))]
pub async fn db_get_all_students(connection: &PgPool) -> Result<Vec<FullStudent>, Error> {
    let query_span = tracing::info_span!("Get students from students table");

    let students = sqlx::query_as!(Student, "select * from students")
        .fetch_all(connection)
        .instrument(query_span)
        .await
        .map_err(|e| {
            Error::new(
                Some(e.to_string()),
                Some("Can not get all students from db".into()),
                ErrorTypes::DbError,
            )
        })?;

    let mut full_students = Vec::with_capacity(students.len());

    for student in students {
        let courses = get_courses(student.id, connection).await?;

        full_students.push(FullStudent {
            id: student.id,
            full_name: student.full_name,
            email: student.email,
            age: student.age,
            img: student.img,
            registration_date: student.registration_date,
            courses,
        });
    }
    Ok(full_students)
}

#[instrument(name = "Delete student from db", skip(connection))]
pub async fn db_delete_student(student_id: Uuid, connection: &PgPool) -> Result<(), Error> {
    //delete all student's courses
    let query_span = tracing::info_span!("Delete courses",%student_id);
    //delete user from students table
    let _ = sqlx::query!("delete from courses where student_id = $1;", student_id)
        .execute(connection)
        .instrument(query_span)
        .await
        .map_err(|e| {
            Error::new(
                Some(e.to_string()),
                Some("Can not find student with the provided id".into()),
                ErrorTypes::DbError,
            )
        })?;

    let query_span = tracing::info_span!("Delete student",%student_id);
    //delete the student from students table
    let _ = sqlx::query!("delete from students where id=$1;", student_id)
        .execute(connection)
        .instrument(query_span)
        .await
        .map_err(|e| {
            Error::new(
                Some(e.to_string()),
                Some("Can not find student with the provided id".into()),
                ErrorTypes::DbError,
            )
        })?;

    Ok(())
}

#[instrument(name = "Adding a new student to db", skip(connection), ret(Debug))]
pub async fn db_insert_new_student(
    data: AddStudent,
    connection: &PgPool,
    avatar_client: Arc<AvatarClient>,
) -> Result<FullStudent, Error> {
    let img = avatar_client.send_request(&data.email).await.unwrap();

    let new_student = FullStudent {
        id: uuid::Uuid::new_v4(),
        full_name: data.full_name.clone(),
        email: data.email,
        age: data.age,
        img,
        registration_date: OffsetDateTime::now_utc(),
        courses: data.courses,
    };

    let query_span = tracing::info_span!("Saving new student in database", id=%new_student.id);

    sqlx::query!(
        r#"
            insert into students (id, full_name, age, registration_date, email, img)
            values ($1, $2, $3, $4, $5, $6);
        "#,
        new_student.id,
        new_student.full_name,
        new_student.age,
        new_student.registration_date,
        new_student.email,
        new_student.img
    )
    .execute(connection)
    .instrument(query_span)
    .await
    .map_err(|e| {
        Error::new(
            Some(e.to_string()),
            Some("Can not insert the student to db".into()),
            ErrorTypes::DbError,
        )
    })?;

    //inser courses
    insert_courses(new_student.id, &new_student.courses, false, connection).await?;

    Ok(new_student)
}

#[instrument(name = "Changing student", skip(connection), ret(Debug))]
pub async fn db_change_student(
    student_id: Uuid,
    data: EditStudent,
    connection: &PgPool,
) -> Result<FullStudent, Error> {
    //check if student exist
    let query_span = tracing::info_span!("Finding student in db", %student_id);
    sqlx::query!("select * from students where id=$1;", student_id)
        .fetch_one(connection)
        .instrument(query_span)
        .await
        .map_err(|e| {
            Error::new(
                Some(e.to_string()),
                Some("Can not find student with the provided id".into()),
                ErrorTypes::DbError,
            )
        })?;

    //insert new data to students table
    sqlx::query!(
        "update students set email=$1, age=$2 where id=$3;",
        data.email,
        data.age,
        student_id
    )
    .execute(connection)
    .await
    .map_err(|e| {
        Error::new(
            Some(e.to_string()),
            Some("Can not set new studet's data to db".into()),
            ErrorTypes::DbError,
        )
    })?;

    //update courses
    insert_courses(student_id, &data.courses, true, connection).await?;
    let result = db_get_student(student_id, connection).await?;
    Ok(result)
}

async fn get_courses(student_id: Uuid, connection: &PgPool) -> Result<Vec<String>, Error> {
    //get student's courses
    let query_span = tracing::info_span!("Get courses",%student_id);
    Ok(sqlx::query!(
        "select course_name from courses where student_id=$1",
        student_id
    )
    .fetch_all(connection)
    .instrument(query_span)
    .await
    .map_err(|e| {
        Error::new(
            Some(e.to_string()),
            Some("Can not find student's courses".into()),
            ErrorTypes::DbError,
        )
    })?
    .into_iter()
    .map(|rec| rec.course_name)
    .collect())
}

async fn insert_courses(
    student_id: Uuid,
    courses: &Vec<String>,
    delete_old: bool,
    connection: &PgPool,
) -> Result<(), Error> {
    //delete all student's courses
    if delete_old {
        sqlx::query!("delete from courses where student_id=$1;", student_id)
            .execute(connection)
            .await
            .map_err(|e| {
                Error::new(
                    Some(e.to_string()),
                    Some("Can not delete student's courses".into()),
                    ErrorTypes::DbError,
                )
            })?;
    }

    //create query
    let mut insert_query = "INSERT INTO courses (student_id,course_name) values ".to_string();
    for item in courses.iter() {
        insert_query.push_str(&format!("('{}','{}'),", student_id, item));
    }
    //delete comma in the end
    insert_query.pop();

    //insert new courses
    let query_span =
        tracing::info_span!("Saving new courses to database",%student_id,courses=?courses);
    sqlx::query(&insert_query)
        .execute(connection)
        .instrument(query_span)
        .await
        .map_err(|e| {
            Error::new(
                Some(e.to_string()),
                Some("Can not insert new student's courses".into()),
                ErrorTypes::DbError,
            )
        })?;

    Ok(())
}
