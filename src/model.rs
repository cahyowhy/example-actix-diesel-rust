use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;

pub const DEFAULT_LIMIT: i64 = 20;

#[derive(Deserialize)]
pub struct Pagination {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub name: String,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct CreateUser {
    name: String,
    email: String,
    #[serde(skip_deserializing)]
    pub username: String,
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

        self.username = combined_name + &ts
    }
}
