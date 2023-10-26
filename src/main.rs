extern crate lazy_static;
use lazy_static::lazy_static;
use crate::configuration::Configuration;

mod configuration;
mod user;
mod user_service;
// mod user_controller;
mod user_client;

lazy_static! {
    static ref CONFIG: Configuration = Configuration::read_from_config_file("resources").unwrap();
}

#[tokio::main]
async fn main() {
    println!("Hello world!");
}
