use crate::{
    app::AppState,
    auth::JwtMiddleware,
    db::{
        db_change_student, db_delete_student, db_get_all_students, db_get_student,
        db_insert_new_student,
    },
    errors::{Error, ErrorTypes},
    schemas::{AddStudent, EditStudent},
};
use actix_web::{delete, get, post, web, HttpRequest, HttpResponse, Responder, ResponseError};

use tracing::instrument;

use uuid::Uuid;
use validator::Validate;

use super::AvatarClient;

#[get("/health_check")]
#[instrument(skip_all, name = "Health check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}


#[get("/")]
#[instrument(skip_all,name="Index page",fields(uri = %req.uri(), method= %req.method()))]
pub async fn index(req: HttpRequest, _: JwtMiddleware) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf8")
        .insert_header(("content-length", 10))
        .body(include_str!("../../templates/index.html"))
}

#[post("/students")]
#[instrument(skip_all,name="Add new student",fields(uri = %req.uri(), method= %req.method(),data=?form))]
pub async fn post_student(
    state: web::Data<AppState>,
    avatar_client: web::Data<AvatarClient>,
    form: web::Json<AddStudent>,
    req: HttpRequest,
    _: JwtMiddleware,
) -> impl Responder {
    //Data validation
    if let Err(error) = form.validate().map_err(|e| {
        Error::new(
            Some(serde_json::to_string_pretty(&e).unwrap()),
            Some("Invalid data".into()),
            ErrorTypes::ValidationError,
        )
    }) {
        tracing::error!("Invalid input data. Errors: {}", error);
        return error.error_response();
    }

    match db_insert_new_student(
        form.into_inner(),
        &state.connection,
        avatar_client.into_inner(),
    )
    .await
    {
        Ok(student) => {
            tracing::info!(
                "Student_id {} - Student's details has been saved",
                student.id,
            );
            HttpResponse::Ok().json(web::Json(student))
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {}", e);
            e.error_response()
        }
    }
}

#[post("/students/change/{id}")]
#[instrument(skip_all,name="Change student",fields(uri = %req.uri(), method= %req.method(),student_id=%id,data=?form))]
pub async fn change_student(
    id: web::Path<Uuid>,
    state: web::Data<AppState>,
    form: web::Json<EditStudent>,
    req: HttpRequest,
    _: JwtMiddleware,
) -> impl Responder {
    //Data validation
    if let Err(error) = form.validate().map_err(|e| {
        Error::new(
            Some(serde_json::to_string_pretty(&e).unwrap()),
            Some("Invalid data".into()),
            ErrorTypes::ValidationError,
        )
    }) {
        tracing::error!("Invalid input data. Errors: {}", error);
        return error.error_response();
    }

    match db_change_student(*id, form.into_inner(), &state.connection).await {
        Ok(student) => {
            tracing::info!("Student_id {} - Student details has been saved", id);
            HttpResponse::Ok().json(student)
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {}", e);
            e.error_response()
        }
    }
}

#[get("/students")]
#[instrument(skip_all,name="Get all students",fields(uri = %req.uri(), method= %req.method()))]
pub async fn get_all_students(
    state: web::Data<AppState>,
    req: HttpRequest,
    _: JwtMiddleware,
) -> impl Responder {
    match db_get_all_students(&state.connection).await {
        Ok(data) => {
            tracing::info!("Successfully get all students");
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            tracing::error!("Failed get all students: {}", e);
            e.error_response()
        }
    }
}

#[get("/students/{student_id}")]
#[instrument(skip(state,req),name="Get student",fields(uri = %req.uri(), method= %req.method()))]
pub async fn get_student(
    student_id: web::Path<Uuid>,
    state: web::Data<AppState>,
    req: HttpRequest,
    _: JwtMiddleware,
) -> impl Responder {
    match db_get_student(*student_id, &state.connection).await {
        Ok(data) => {
            tracing::info!("Successfully get student with id: '{}'", student_id);
            HttpResponse::Ok().json(data)
        }
        Err(e) => {
            tracing::error!("Failed get student: {}", e);
            e.error_response()
        }
    }
}

#[get("/students/{student_id}/avatar")]
#[instrument(skip(state,req),name="Get student's avatar",fields(uri = %req.uri(), method= %req.method()))]
pub async fn get_avatar(
    student_id: web::Path<Uuid>,
    state: web::Data<AppState>,
    req: HttpRequest,
    _: JwtMiddleware,
) -> impl Responder {
    match db_get_student(*student_id, &state.connection).await {
        Ok(data) => {
            tracing::info!(
                "Successfully get student's avatar with id: '{}'",
                student_id
            );
            HttpResponse::Ok().json(data.img)
        }
        Err(e) => {
            tracing::error!("Failed get student's avatar: {}", e);
            e.error_response()
        }
    }
}

#[delete("/delete/{student_id}")]
#[instrument(skip(state,req),name="Delete student",fields(uri = %req.uri(), method= %req.method()))]
pub async fn delete_student(
    student_id: web::Path<Uuid>,
    state: web::Data<AppState>,
    req: HttpRequest,
    _: JwtMiddleware,
) -> impl Responder {
    match db_delete_student(*student_id, &state.connection).await {
        Ok(_) => {
            tracing::info!("Successfully delete student with id: '{}'", student_id);
            HttpResponse::Ok().json(format!("{{'Deleted student':{} }}", student_id))
        }
        Err(e) => {
            tracing::error!("Failed get student's avatar: {}", e);
            e.error_response()
        }
    }
}
