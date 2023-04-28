use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use lazy_static::lazy_static;
use regex::Regex;
use validator::{Validate, ValidationError};

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct Student {
    pub id: Uuid,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub email: String,
    pub age: i32,
    pub img: String,
    pub registration_date: OffsetDateTime,
}

//regex for name falidation. Must contains letters and space
lazy_static! {
    static ref FULLNAME_REGEX: Regex =
        Regex::new(r"\b\w{3,}\D\b\s{1}\b\w{3,}\D\b$").expect("Ivalid regular expression");
    static ref COURSES_REGEX: Regex =
        Regex::new(r"\b[a-zA-Z]{2,}[.\\\d]{0,}\b").expect("Ivalid regular expression");
}

//User from Json with validation
#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct AddStudent {
    #[validate(regex(
        path = "FULLNAME_REGEX",
        message = "Must contais only letters and space!"
    ))]
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(range(min = 16, max = 120))]
    pub age: i32,
    #[validate(custom = "courses_validation")]
    pub courses: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FullStudent {
    pub id: Uuid,
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub email: String,
    pub age: i32,
    pub img: String,
    #[serde(rename = "registrationDate")]
    pub registration_date: OffsetDateTime,
    pub courses: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct EditStudent {
    #[validate(email)]
    pub email: String,
    #[validate(range(min = 16, max = 120))]
    pub age: i32,
    #[validate(custom = "courses_validation")]
    pub courses: Vec<String>,
}

fn courses_validation(courses: &Vec<String>) -> Result<(), ValidationError> {
    for c in courses {
        if !COURSES_REGEX.is_match(c) {
            let mut error = ValidationError::new("Invalid courses data");
            error.add_param("incorrect course".into(), c);
            return Err(error);
        }
    }
    Ok(())
}
