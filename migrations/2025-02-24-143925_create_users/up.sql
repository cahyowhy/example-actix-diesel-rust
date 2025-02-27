-- Your SQL goes here
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  username VARCHAR(25) NOT NULL,
  email TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL
)