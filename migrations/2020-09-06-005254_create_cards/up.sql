-- Your SQL goes here
CREATE TABLE cards (
    id SERIAL PRIMARY KEY,
    phrase TEXT UNIQUE NOT NULL,
    images TEXT[] NOT NULL
);
CREATE TABLE user_cards (
    id SERIAL PRIMARY KEY,
    user_id SERIAL,
    phrase TEXT NOT NULL,
    images TEXT[] NOT NULL,
    categories TEXT[] NOT NULL,
    UNIQUE (user_id, phrase),
    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
            REFERENCES users(id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
