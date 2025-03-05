-- Your SQL goes here
TRUNCATE TABLE users RESTART IDENTITY;
ALTER TABLE users 
    ADD COLUMN password TEXT NOT NULL,
    ADD COLUMN image_profile TEXT;