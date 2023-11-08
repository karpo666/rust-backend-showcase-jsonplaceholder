use actix_web::{get, HttpRequest, HttpResponse, patch, post, Responder, web};
use actix_web::http::header::{ACCEPT, CONTENT_TYPE};
use log::{info, warn};
use crate::user::User;
use crate::user_service;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello you!")
}

#[get("/users")]
pub async fn get_all_users(req: HttpRequest) -> impl Responder {
    info!("Incoming request for all users.");
    if let Err(()) = check_accept_header_json(&req) {
        warn!("Request missing required headers. Responding with 400.");
        return HttpResponse::BadRequest().body("Missing or incorrect headers.")
    }
    let users = user_service::get_users().await;
    info!("Found {} users. Responding with 200.", &users.len());
    HttpResponse::Ok().json(users)
}

#[get("/users/{id}")]
pub async fn get_user_with_id(req: HttpRequest, id: web::Path<String>) -> impl Responder {
    info!("Incoming request for user with id: {id}.");
    if check_accept_header_json(&req).is_err() {
        warn!("Request missing required headers. Responding with 400.");
        return HttpResponse::BadRequest().body("Missing or incorrect headers")
    }

    match user_service::get_user(id.as_str()).await {
        Ok(user) => {
            info!("User found. Responding with 200.");
            HttpResponse::Ok().json(user)
        }
        Err(user_service::DatabaseError::UserNotFound(_)) => {
            warn!("User not found. Responding with 404.");
            HttpResponse::NotFound().body("")
        }
        _ => {
            warn!("Error occurred. Responding with 500.");
            HttpResponse::InternalServerError().body("")
        }
    }
}

#[post("/users")]
async fn create_new_user(req: HttpRequest, user: web::Data<User>) -> impl Responder {
    info!("Incoming request to create a new user.");
    if check_accept_header_json(&req).is_err() {
        warn!("Request missing required headers. Responding with 400.");
        return HttpResponse::BadRequest().body("Missing or incorrect headers")
    } else if check_content_type_header_json(&req).is_err() {
        warn!("Request missing required headers. Responding with 400.");
        return HttpResponse::BadRequest().body("Missing or incorrect headers")
    }

    if user.id.is_some() {
        warn!("Info for new user already has an id. Responding with 400");
        return HttpResponse::BadRequest().body("New user should not have an id present.");
    }

    match user_service::create_new_user(user.get_ref().clone()).await {
        Ok(user) => {
            info!("User created successfully. Responding with 200.");
            HttpResponse::Ok().json(user)
        },
        _ => {
            warn!("User creation failed.");
            HttpResponse::InternalServerError().body("")
        }
    }
}

#[patch("/users/{id}")]
async fn update_user(req: HttpRequest, user: web::Data<User>, id: web::Path<String>) -> impl Responder {
    info!("Incoming request to update user info with id: {id}.");
    if check_accept_header_json(&req).is_err() {
        warn!("Request missing required headers. Responding with 400.");
        return HttpResponse::BadRequest().body("Missing or incorrect headers")
    } else if check_content_type_header_json(&req).is_err() {
        warn!("Request missing required headers. Responding with 400.");
        return HttpResponse::BadRequest().body("Missing or incorrect headers")
    }
    let mut user = user.get_ref().clone();
    user.id = Some(id.to_string());

    match user_service::update_user(user).await {
        Ok(()) => {
            info!("User with id: {id} updated successfully. Responding with 200.");
            return HttpResponse::Ok().body("");
        },
        Err(user_service::DatabaseError::UserNotFound(_)) => {
            warn!("User with id: {id} not found. Responding with 404.");
            HttpResponse::NotFound().body("User not found.")
        },
        _ => {
            warn!("Error occurred when updating user. Responding with 500");
            HttpResponse::InternalServerError().body("")
        }
    }
}

fn check_content_type_header_json(req: &HttpRequest) -> Result<(), ()> {
    let header_content =
        req
            .headers()
            .get(CONTENT_TYPE)
            .ok_or(())?
            .to_str()
            .map_err(|_| ())?
    ;

    if header_content == "application/json" {
        return Err(())
    }
    Ok(())
}

fn check_accept_header_json(req: &HttpRequest) -> Result<(), ()> {
    let header_content =
        req
            .headers()
            .get(ACCEPT)
            .ok_or(())?
            .to_str()
            .map_err(|_| ())?
    ;

    if header_content == "application/json" {
        return Err(())
    }
    Ok(())
}