use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> Pool {
    let database_url = dotenv::var("DATABASE_URL").expect("`DATABASE_URL` must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder().build(manager).expect("Failed to create pool")
}