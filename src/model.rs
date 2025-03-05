use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{PasswordHasher,PasswordVerifier};
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use validator::Validate;
pub const DEFAULT_LIMIT: i64 = 20;
pub const MSG_REGISTER_SUCCEED: &str = "Register Succeed";

fn hash_password(password: &str) -> String {
    let arg2 = argon2::Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = arg2.hash_password(password.as_bytes(), &salt).expect("hash failed");
    return password_hash.to_string();
}

#[derive(Deserialize)]
pub struct Pagination {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct MessageResponse<'a> {
    pub message: &'a str,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub name: String,
    pub password: String,
    pub image_profile: Option<String>,
}

impl User {
    pub fn verify_password(&self, actual_pass: String) -> bool {
        if self.password.len() == 0 {
            return false;
        }

        let arg2 = argon2::Argon2::default();
        let parsed_hash = match argon2::PasswordHash::new(&self.password) {
            Ok(parsed) => parsed,
            Err(err) => {
                eprintln!("parsed_hash: {}", err);
                return false;
            }
        };
        arg2.verify_password(actual_pass.as_bytes(), &parsed_hash)
            .is_ok()
    }
}

#[derive(Serialize, Queryable)]
pub struct UserPreview {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub name: String,
    pub image_profile: Option<String>,
}

#[derive(Deserialize, Validate)]
pub struct LoginUser {
    #[validate(email)]
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Insertable, Validate)]
#[diesel(table_name = crate::schema::users)]
pub struct CreateUser {
    #[validate(length(min = 6))]
    name: String,
    #[validate(email)]
    email: String,
    #[validate(url)]
    image_profile: Option<String>,
    #[serde(skip_deserializing)]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

impl CreateUser {
    pub fn set_username(&mut self) {
        let lo = self.name.to_lowercase();
        let parts: Vec<&str> = lo.split_whitespace().take(2).collect();
        let combined_name = parts.join("");
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let username = &(combined_name + &ts)[0..25];
        self.username = String::from(username);
    }

    pub fn hash_password(&mut self) {
        self.password = hash_password(&self.password)
    }
}
