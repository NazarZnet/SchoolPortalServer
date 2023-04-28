# School server with Rust and Actix Web
Written only with educational purpose

## Getting Started
First off all make sure you have [Docker](https://www.docker.com/) and [Docker Compose](https://docs.docker.com/compose/) installed. You can run the server locally with:
```
git clone https://github.com/NazarZnet/SchoolPortalServer.git
cd SchoolPortalServer
docker compose up
```
Then send requests to http://127.0.0.1:8000

## Key Technologies:
- [Rust](https://www.rust-lang.org/)
- [Actix Web](https://actix.rs/)
- [JWT](https://jwt.io/)
- [Sqlx](https://crates.io/crates/sqlx)
- [Docker](https://www.docker.com/)

## API Documentation

|            **URI**            | **METHOD** |                                               **DESCRIPTION**                                               |
|:-----------------------------:|:----------:|:-----------------------------------------------------------------------------------------------------------:|
|               /               |     GET    | Returns main HTML page                                                                                      |
|          /auth/signup         |    POST    | Register new user. Send username, email and password in JSON format. Returns created user                   |
|          /auth/login          |    POST    | User log in. Send email and password in JSON format. Returns operation status, access and refresh tokens    |
|          /auth/logout         |     GET    | User log out. Returns operation status                                                                      |
|         /auth/refresh         |     GET    | Refresh authorization. Returns status and new access token                                                  |
|           /students           |     GET    | Returns all existed students                                                                                |
|     /students/{student_id}    |     GET    | Returns a student with the id                                                                               |
| /students/{student_id}/avatar |     GET    | Returns student's avatar                                                                                    |
|           /students           |    POST    | Create a new student. Send fullName, email, age and list of courses in JSON format. Returns created student |
| /students/change/{student_id} |    POST    | Change a student. Send new email, age and list of courses.  Returns changed student                         |
|      /delete/{student_id}     |   DELETE   | Delete a student with provided id. Returns deleted student's id                                             |




