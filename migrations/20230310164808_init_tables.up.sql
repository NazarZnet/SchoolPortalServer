-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE IF NOT EXISTS students(
    id UUID NOT NULL UNIQUE PRIMARY KEY DEFAULT uuid_generate_v4(),
    full_name VARCHAR(255) NOT NULL UNIQUE,
    age INTEGER NOT NULL,
    registration_date TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS courses(
    id SERIAL PRIMARY KEY,
    student_id UUID NOT NULL,
    course_name TEXT NOT NULL,
    FOREIGN KEY (student_id) REFERENCES students(id)
);