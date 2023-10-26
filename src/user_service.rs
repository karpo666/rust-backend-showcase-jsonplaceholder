use mongodb::{Client, Collection};
use mongodb::bson::{Bson, doc};
use mongodb::options::{ClientOptions};
use crate::user::User;

#[derive(Eq, PartialEq, Debug)]
enum DatabaseError {
    UserNotFound(String),
    MongoConnectionFailed,
    OperationFailed
}

/// Get MongoDB collection based on the given configuration.
async fn get_collection(connection_string: String, database_name: String) -> Result<Collection<User>, DatabaseError> {
    // Parse options and attempt connection.
    let client_options = match ClientOptions::parse(connection_string).await {
        Ok(options) => options,
        _ => return Err(DatabaseError::MongoConnectionFailed)
    };

    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        _ => return Err(DatabaseError::MongoConnectionFailed)
    };

    Ok(client.database(database_name.as_str()).collection("users"))
}

async fn get_user_from_db(id: String, connection_string: String, database_name: String) -> Result<User, DatabaseError> {
    let collection = get_collection(connection_string, database_name).await?;
    let user_result = collection.find_one(
        doc! {
            "id": &id
        },
        None
    ).await;

    let user_option = match user_result {
        Ok(option) => option,
        _ => return Err(DatabaseError::UserNotFound(id))
    };

    match user_option {
        Some(user) => Ok(user),
        None => Err(DatabaseError::UserNotFound(id))
    }
}

async fn add_user_to_db(user: User, database_name: String, collection_name: String) -> Result<Bson, DatabaseError> {
    let collection = get_collection(database_name, collection_name).await?;
    let insert_result_result = collection.insert_one(user, None).await;

    match insert_result_result {
        Ok(insert_result) => Ok(insert_result.inserted_id),
        Err(_) => Err(DatabaseError::OperationFailed)
    }
}

#[cfg(test)]
mod test {
    use testcontainers::{clients, GenericImage};
    use testcontainers::core::WaitFor;
    use crate::configuration;
    use super::*;

    #[tokio::test]
    async fn test_get_collection_no_connection() {
        let collection_result = get_collection("t".to_string(), "t".to_string()).await;
        assert!(collection_result.is_err());
    }

    #[tokio::test]
    async fn test_get_collection() {
        setup_test_container();

        let configuration = configuration::Configuration::read_from_config_file("resources/test/config.toml").unwrap();
        let collection_result =
            get_collection(configuration.database.url, "showcase_test".to_string()).await
        ;

        assert!(collection_result.is_ok());

        let collection = collection_result.unwrap();
        assert_eq!("users", collection.name());
    }

    #[tokio::test]
    async fn test_get_user_from_database_not_found() {
        setup_test_container();
        let configuration = configuration::Configuration::read_from_config_file("resources/test/config.toml").unwrap();
        assert_eq!(
            Err(DatabaseError::UserNotFound("666".to_string())),
            get_user_from_db("666".to_string(), configuration.database.url, "showcase_test".to_string()).await
        );
    }

    fn setup_test_container() {
        let msg = WaitFor::message_on_stdout("Waiting for connections");
        let mongo_image =
            GenericImage::new("mongo", "latest")
                .with_wait_for(msg)
                .with_env_var("MONGO_INITDB_DATABASE", "showcase_test")
                .with_env_var("MONGO_INITDB_USERNAME", "test_root")
                .with_env_var("MONGO_INITDB_PASSWORD", "test_pass")
                .with_exposed_port(27017)
        ;

        clients::Cli::default().run(mongo_image);
    }
}