CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL
);
CREATE TABLE IF NOT EXISTS user_auths (
    id SERIAL PRIMARY KEY,
    user_id SERIAL UNIQUE,
    password_hash TEXT NOT NULL,
    salt TEXT NOT NULL,
    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
            REFERENCES users(id)
            ON DELETE CASCADE
            ON UPDATE CASCADE
);
ALTER TABLE IF EXISTS cards
RENAME TO tiles;
CREATE TABLE IF NOT EXISTS tiles (
    id SERIAL PRIMARY KEY,
    phrase TEXT UNIQUE NOT NULL,
    images TEXT[] NOT NULL
);
ALTER TABLE IF EXISTS user_cards
RENAME TO user_tiles;
CREATE TABLE IF NOT EXISTS user_tiles (
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
