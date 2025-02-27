use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use diesel::{
    query_dsl::methods::{FindDsl, LimitDsl, OffsetDsl},
    RunQueryDsl,
};
use dotenv::dotenv;
use schema::users;

mod db;
mod model;
pub mod schema;

async fn create_user(
    pool: web::Data<db::Pool>,
    new_user: web::Json<model::CreateUser>,
) -> impl Responder {
    use self::users::dsl::*;

    let mut new_user_val = new_user.into_inner();
    new_user_val.set_username();

    let mut conn = pool.get().expect("Couldn't get a database connection");
    let result = diesel::insert_into(users)
        .values(&new_user_val)
        .get_result::<model::User>(&mut conn);

    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::InternalServerError().body("failed create new user"),
    }
}

async fn get_users(
    pool: web::Data<db::Pool>,
    paging: web::Query<model::Pagination>,
) -> impl Responder {
    use self::users::dsl::*;
    let offset = paging.offset.unwrap_or(0);
    let limit = paging.limit.unwrap_or(model::DEFAULT_LIMIT);

    let mut conn = pool.get().expect("Couldn't get a database connection");
    let results = users
        .limit(limit)
        .offset(offset)
        .load::<model::User>(&mut conn);
    match results {
        Ok(user_lists) => HttpResponse::Ok().json(user_lists),
        Err(_) => HttpResponse::InternalServerError().body("failed load users"),
    }
}

async fn get_user(pool: web::Data<db::Pool>, user_id: web::Path<i32>) -> impl Responder {
    use self::users::dsl::*;
    let mut conn = pool.get().expect("Couldn't get a database connection");
    let result = users
        .find(user_id.into_inner())
        .first::<model::User>(&mut conn);

    match result {
        Ok(user_dat) => HttpResponse::Ok().json(user_dat),
        Err(_) => HttpResponse::InternalServerError().body("failed load user"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let pool = db::establish_connection();
    let db_con = web::Data::new(pool.clone());
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::clone(&db_con))
            .route("/users", web::post().to(create_user))
            .route("/users", web::get().to(get_users))
            .route("/user/{id}", web::get().to(get_user))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
