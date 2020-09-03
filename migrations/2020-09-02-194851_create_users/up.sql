-- Your SQL goes here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE
);
CREATE TABLE user_auths (
    id SERIAL PRIMARY KEY,
    user_id SERIAL UNIQUE,
    password_hash TEXT NOT NULL,
    salt TEXT NOT NULL
);
