extern crate lazy_static;

use actix_web::{App, HttpServer};
use lazy_static::lazy_static;
use log::{info};
use crate::configuration::Configuration;

mod configuration;
mod user;
mod user_service;
mod user_controller;
mod user_client;

lazy_static! {
    static ref CONFIG: Configuration =
        Configuration::read_from_config_file("resources/config.toml").unwrap()
    ;
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("Starting rust-backend-showcase...");
    info!("Listening on port 8080.");
    println!();

    HttpServer::new(|| {
        App::new()
            .service(user_controller::hello)
            .service(user_controller::get_all_users)
            .service(user_controller::get_user_with_id)
            .service(user_controller::create_new_user)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
