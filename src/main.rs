use std::collections::HashMap;

use crate::schema::users::dsl::*;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    http::header::Accept,
    middleware::{from_fn, Next},
    web::{self, Header, Query},
    App, Error, HttpMessage, HttpResponse, HttpServer, Responder,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use dotenv::dotenv;
use jsonwebtoken::decode;
use model::{Claims, MSG_REGISTER_SUCCEED};
use validator::Validate;

mod db;
mod model;
pub mod schema;

async fn authenticate_middleware(
    _: Header<Accept>,
    _: Query<HashMap<String, String>>,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if req.path() == "/users/login" {
        let res = next.call(req).await;
        return res; 
    }

    if let Some(auth_header) = req.headers().get("Authorization") {
        let auth_str = auth_header.to_str().unwrap_or("").replace("Bearer ", "");

        println!("{}", auth_str);
        let claims = decode::<Claims>(
            &auth_str,
            &model::KEYS.decoding,
            &jsonwebtoken::Validation::default(),
        );
        match claims {
            Ok(dat) => {
                req.extensions_mut().insert(dat);
                let res = next.call(req).await;
                res
            }
            Err(err) => Err(actix_web::error::ErrorUnauthorized(format!(
                "unauthorized {}",
                err
            ))),
        }
    } else {
        Err(actix_web::error::ErrorBadRequest(
            "Authorization header empty",
        ))
    }
}

async fn login_user(
    pool: web::Data<db::Pool>,
    loggedin_user: web::Json<model::LoginUser>,
) -> impl Responder {
    let val = loggedin_user.into_inner();
    let mut conn = pool.get().expect("Couldn't get a database connection");
    let result = users
        .filter(email.eq(&val.email)) // use &val.email
        .first::<model::User>(&mut conn);

    match result {
        Ok(fetch_user) => {
            if fetch_user.verify_password(val.password) {
                // use &val.password
                HttpResponse::Ok().json(fetch_user.get_claim_jwt())
            } else {
                HttpResponse::Unauthorized().json(model::MessageResponse {
                    message: "Unauthorized",
                })
            }
        }
        Err(_) => HttpResponse::NotFound().json(model::MessageResponse {
            message: "User not found",
        }),
    }
}

async fn create_user(
    pool: web::Data<db::Pool>,
    new_user: web::Json<model::CreateUser>,
) -> impl Responder {
    let mut new_user_val = new_user.into_inner();
    new_user_val.set_username();
    new_user_val.hash_password();

    let vld_result = new_user_val.validate();
    if let Err(err) = vld_result {
        return HttpResponse::BadRequest().json(model::MessageResponse {
            message: format!("validation error {}", err).as_str(),
        });
    }

    let mut conn = pool.get().expect("Couldn't get a database connection");
    let result = diesel::insert_into(users)
        .values(&new_user_val)
        .execute(&mut conn);

    match result {
        Ok(_) => HttpResponse::Ok().json(model::MessageResponse {
            message: MSG_REGISTER_SUCCEED,
        }),
        Err(e) => HttpResponse::InternalServerError().json(model::MessageResponse {
            message: format!("failed create new user {}", e).as_str(),
        }),
    }
}

async fn get_users(
    pool: web::Data<db::Pool>,
    paging: web::Query<model::Pagination>,
) -> impl Responder {
    let offset = paging.offset.unwrap_or(0);
    let limit = paging.limit.unwrap_or(model::DEFAULT_LIMIT);

    let mut conn = pool.get().expect("Couldn't get a database connection");
    let results = users
        .select((id, username, email, name, image_profile))
        .limit(limit)
        .offset(offset)
        .load::<model::UserPreview>(&mut conn);
    match results {
        Ok(user_lists) => HttpResponse::Ok().json(user_lists),
        Err(e) => HttpResponse::InternalServerError().json(model::MessageResponse {
            message: format!("failed get users {}", e).as_str(),
        }),
    }
}

async fn get_user(pool: web::Data<db::Pool>, user_id: web::Path<i32>) -> impl Responder {
    let mut conn = pool.get().expect("Couldn't get a database connection");
    let result = users
        .select((id, username, email, name, image_profile))
        .find(user_id.into_inner())
        .first::<model::UserPreview>(&mut conn);

    match result {
        Ok(user_dat) => HttpResponse::Ok().json(user_dat),
        Err(e) => HttpResponse::InternalServerError().json(model::MessageResponse {
            message: format!("failed get user {}", e).as_str(),
        }),
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
            .wrap(from_fn(authenticate_middleware))
            .route("/users", web::post().to(create_user))
            .route("/users", web::get().to(get_users))
            .route("/user/{id}", web::get().to(get_user))
            .route("/users/login", web::post().to(login_user))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
