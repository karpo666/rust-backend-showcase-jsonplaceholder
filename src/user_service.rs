use log::{info, warn};
use mongodb::{Client, Collection};
use mongodb::bson::{doc, Document};
use mongodb::options::ClientOptions;
use crate::{CONFIG, user_client};
use crate::user::User;
use crate::user_client::UserClientError;

#[derive(Eq, PartialEq, Debug)]
pub enum DatabaseError {
    UserNotFound(String),
    MongoConnectionFailed,
    OperationFailed
}

/// Get all users across the database and JsonPlaceholder.
///
/// ## Returns.
/// Vector containing all found users.
pub async fn get_users() -> Vec<User> {
    let database_result =
        get_all_users_from_db_with_config(
            &CONFIG.database.url,
            &CONFIG.database.database_name
        ).await
    ;

    let mut users = match database_result {
        Ok(users) => users,
        _ => vec![]
    };

    let database_ids: Vec<String> = users.clone().iter().map(|user| user.id.clone().unwrap()).collect();

    let jph_users = match user_client::get_users().await {
        Ok(users) => users,
        _ => vec![]
    };

    jph_users.iter().for_each(|user| {
        if !database_ids.contains(&user.id.clone().unwrap()) {
            users.push(user.clone());
        };
    });

    users
}

/// Create a new user.
///
/// ## Arguments.
/// * `user` - New user info without id.
///
/// ## Returns.
/// A result containing the user info enriched with id or an error.
pub async fn create_new_user(mut user: User) -> Result<User, DatabaseError> {
    let creation_result =
        create_user_to_db_with_config(
            &mut user,
            &CONFIG.database.url,
            &CONFIG.database.database_name
        ).await
    ;

    if creation_result.is_err() {
        return Err(DatabaseError::OperationFailed)
    };

    user.id = Some(creation_result.unwrap());

    Ok(user)
}

/// Get user with specific id.
///
/// ## Arguments.
/// * `id` - User id.
///
/// ## Returns.
/// A result containing the found user or an occurred error.
pub async fn get_user(id: &str) -> Result<User, DatabaseError> {
    let database_result =
        get_user_from_db_with_config(
            id,
            &CONFIG.database.url,
            &CONFIG.database.database_name
        ).await
    ;

    match database_result {
        Ok(user) => return Ok(user),
        Err(DatabaseError::MongoConnectionFailed) => info!("Could not establish connection with mongoDB!"),
        Err(DatabaseError::UserNotFound(_)) => info!("Could not find user in mongoDB, attempting JsonPlaceholder!"),
        _ => warn!("Error occurred when searching user from mongoDB")
    }

    info!("Checking JsonPlaceholder for user with id: {id}");
    let jph_result = user_client::get_user(id.to_string()).await;

    return match jph_result {
        Ok(user) => Ok(user),
        Err(UserClientError::UserNotFound(_)) => Err(DatabaseError::UserNotFound(id.to_string())),
        _ => Err(DatabaseError::OperationFailed)
    }
}

/// Update user info.
///
/// ## Arguments.
/// * `user` - Updated user info.
///
/// ## Returns.
/// Result with an empty `OK` or an error.
pub async fn update_user(user: User) -> Result<(), DatabaseError> {
    update_user_in_db_with_config(
        user,
        &CONFIG.database.url,
        &CONFIG.database.database_name
    ).await
}

/// Get MongoDB collection based on the given configuration.
///
/// ## Arguments.
/// * `connection_string` - Connection string that will be used to connect to MongoDB. Should contain username and password.
/// * `database_name` - Database we are using.
///
/// ## Returns.
/// A result containing possible `DatabaseError` or the collection with name "users".
async fn get_user_collection(connection_string: &str, database_name: &str) -> Result<Collection<User>, DatabaseError> {
    // Parse options and attempt connection.
    let client_options = match ClientOptions::parse(connection_string).await {
        Ok(options) => options,
        _ => return Err(DatabaseError::MongoConnectionFailed)
    };

    // Create client.
    let client = match Client::with_options(client_options) {
        Ok(client) => client,
        _ => return Err(DatabaseError::MongoConnectionFailed)
    };

    // Get collection.
    let collection_name = "users";
    Ok(client.database(database_name).collection(collection_name))
}

/// Get all users from database.
///
/// ## Arguments.
/// * `connection_string` - Connection string that will be used to connect to MongoDB. Should contain username and password.
/// * `database_name` - Database we are using.
///
/// ## Returns.
/// A result containing either a  vector consisting of the fetched users or an error.
async fn get_all_users_from_db_with_config(connection_string: &str, database_name: &str) -> Result<Vec<User>, DatabaseError> {
    // Get collection.
    let collection = get_user_collection(connection_string, database_name).await?;

    // Get users.
    let mut cursor = match collection.find(None, None).await {
        Ok(c) => c,
        Err(e) => {
            println!("{:?}", e);
            return Err(DatabaseError::OperationFailed)
        }
    };

    // Iterate through found users and parse them from `RawDocument` to `User`.
    // Return resulting vector.
    let mut result= vec![];
    while cursor.advance().await.map_err(|_| DatabaseError::OperationFailed)? {
        let current: User = bson::from_slice(cursor.current().as_bytes()).unwrap();
        result.push(current);
    }

    return Ok(result)
}

/// Get user info from database with id.
///
/// ## Arguments.
/// * `id` - User id.
/// * `connection_string` - Connection string that will be used to connect to MongoDB. Should contain username and password.
/// * `database_name` - Database we are using.
///
/// ## Returns.
/// A result containing possible `DatabaseError` or the user info with given `id`.
async fn get_user_from_db_with_config(id: &str, connection_string: &str, database_name: &str) -> Result<User, DatabaseError> {
    // Get collection.
    let collection = get_user_collection(connection_string, database_name).await?;

    // Do query with given filter.
    let user_result = collection.find_one(
        doc! {
            "id": id
        },
        None
    ).await;

    // Handle and return result.
    let user_option = match user_result {
        Ok(option) => option,
        Err(e) => {
            println!("{:?}", e);
            return Err(DatabaseError::UserNotFound(id.to_string()))
        }
    };

    match user_option {
        Some(user) => Ok(user),
        None => Err(DatabaseError::UserNotFound(id.to_string()))
    }
}

/// Add new user info to database.
///
/// ## Arguments.
/// * `user` - New user info.
/// * `connection_string` - Connection string that will be used to connect to MongoDB. Should contain username and password.
/// * `database_name` - Database we are using.
///
/// # Returns.
/// A result containing possible `DatabaseError` or the new id generated by MongoDB.
async fn create_user_to_db_with_config(user: &mut User, connection_string: &str, database_name: &str) -> Result<String, DatabaseError> {
    // Get collection.
    let collection = get_user_collection(connection_string, database_name).await?;
    let new_id = (get_users_count_with_config(connection_string, database_name).await? as i32 + 101).to_string();

    // Set a new id for user.
    user.id = Some(new_id.clone());

    // Insert new user.
    let insert_result = collection.insert_one(user, None).await;


    // Handle and return result.
    match insert_result {
        Ok(_) => Ok(new_id),
        Err(_) => Err(DatabaseError::OperationFailed)
    }
}

/// Update user info in database.
///
/// ## Arguments.
/// * `user` - Updated user info.
/// * `connection_string` - Connection string that will be used to connect to MongoDB. Should contain username and password.
/// * `database_name` - Database we are using.
///
/// # Returns.
/// Result containing an empty `Ok` or an error.
async fn update_user_in_db_with_config(user: User, connection_string: &str, database_name: &str) -> Result<(), DatabaseError> {
    // Get collection.
    let collection = get_user_collection(connection_string, database_name).await?;
    // Fetch info stored in the database.
    let existing_user =
        get_user_from_db_with_config(
            user.id.clone().unwrap().as_str(),
            connection_string,
            database_name
        ).await?
    ;

    // Update user info.
    let update_result = collection.update_one(
        doc! {
            "id": user.id.clone().unwrap()
        },
        generate_update_document(existing_user, user),
        None
    ).await;

    // Return result.
    match update_result {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{:?}", e);
            Err(DatabaseError::OperationFailed)
        }
    }
}


/// Generate an update `Document` based on the differences between given users.
///
/// ## Arguments.
/// * `original_user` - The original user we are comparing against.
/// * `updated_user` - User with changes made to it.
///
/// ## Returns.
/// A `Document` containing changes and a `set`-command.
fn generate_update_document(original_user: User, updated_user: User) -> Document {
    let mut update_document = doc! {};

    // Serialize the original and update structs to JSON value.
    let original_json = serde_json::to_value(original_user).unwrap();
    let updated_json = serde_json::to_value(updated_user).unwrap();

    // Iterate over the fields and compare.
    for (field, updated_value) in updated_json.as_object().unwrap() {
        if let Some(original_value) = original_json.get(&field) {
            // Skip if values match.
            if &original_value == &updated_value {
                continue
            }
        }
        update_document.insert("$set", doc! { field: bson::to_bson(updated_value).unwrap() });
    }

    update_document
}

/// Delete user with given id from database.
///
/// ## Arguments.
/// * `id` - Id for the user to be deleted.
/// * `connection_string` - Connection string that will be used to connect to MongoDB. Should contain username and password.
/// * `database_name` - Database we are using.
///
/// # Returns.
/// A result containing possible `DatabaseError` an `Ok(())` if everything goes as it should.
async fn remove_user_from_db_with_config(id: &str, connection_string: &str, database_name: &str) -> Result<(), DatabaseError> {
    // Get collection.
    let collection = get_user_collection(connection_string, database_name).await?;

    // Delete user.
    let result = collection.delete_one(
        doc! {
                "id": id
        },
        None
    ).await;

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(DatabaseError::OperationFailed)
    }
}

/// Get document count from database.
///
/// ## Arguments.
/// * `connection_string` - Connection string that will be used to connect to MongoDB. Should contain username and password.
/// * `database_name` - Database we are using.
///
/// # Returns.
/// A result containing possible `DatabaseError` or the count of users in MongoDB.
async fn get_users_count_with_config(database_name: &str, collection_name: &str) -> Result<u64, DatabaseError> {
    // Get collection.
    let collection = get_user_collection(database_name, collection_name).await?;

    // Get user count.
    match collection.count_documents(None, None).await {
        Ok(count) => Ok(count),
        _ => Err(DatabaseError::MongoConnectionFailed)
    }
}

#[cfg(test)]
mod test {
    use testcontainers::GenericImage;
    use testcontainers::clients::Cli;
    use super::*;

    // Database name used in tests.
    const DB_NAME: &str = "showcase_test";
    // Connection string -template.
    const C_STRING: &str = "mongodb://localhost:";

    #[tokio::test]
    async fn test_get_collection_faulty_connection_string() {
        assert!(get_user_collection(&"NOT_URL".to_string(), &"LOL".to_string()).await.is_err());
    }

    #[tokio::test]
    async fn test_get_collection() {
        let client = Cli::default();
        let container = client.run(get_mongo_image());

        let port = container.get_host_port_ipv4(27017);
        let connection_string = format!("{}{}", C_STRING, port);

        let collection_result =
            get_user_collection(&connection_string, DB_NAME).await
        ;

        assert!(collection_result.is_ok());

        let collection = collection_result.unwrap();
        assert_eq!("users", collection.name());

        container.stop();
    }

    #[tokio::test]
    async fn test_get_user_from_database_not_found() {
        let client = Cli::default();
        let container = client.run(get_mongo_image());

        let port = container.get_host_port_ipv4(27017);
        let connection_string = format!("{}{}", C_STRING, port);

        assert_eq!(
            Err(DatabaseError::UserNotFound("666".to_string())),

            get_user_from_db_with_config(
                &"666".to_string(),
                &connection_string,
                DB_NAME
            ).await
        );

        container.stop();
    }

    #[tokio::test]
    async fn test_add_and_get_user_and_get_all_users_from_database() {
        let client = Cli::default();
        let container = client.run(get_mongo_image());

        let port = container.get_host_port_ipv4(27017);
        let connection_string = format!("{}{}", C_STRING, port);

        let insert_result =
            create_user_to_db_with_config(
                &mut User::create_test_user(None),
                &connection_string,
                DB_NAME
            ).await
        ;

        assert!(insert_result.is_ok());
        let inserted_id = insert_result.unwrap();

        let search_result =
            get_user_from_db_with_config(
                &inserted_id,
                &connection_string,
               DB_NAME
            ).await
        ;

        assert!(search_result.is_ok());
        let user = search_result.unwrap();

        assert_eq!(&inserted_id, &user.id.unwrap());

        let get_all_result =
            get_all_users_from_db_with_config(
                &connection_string,
                DB_NAME
            ).await
        ;

        assert!(get_all_result.is_ok());
        let user_list = get_all_result.unwrap();
        assert!(!user_list.is_empty());

        assert_eq!(inserted_id, user_list.get(0).cloned().unwrap().id.unwrap());

        container.stop();
    }

    #[tokio::test]
    async fn test_add_and_get_and_remove_user_from_database() {
        let client = Cli::default();
        let container = client.run(get_mongo_image());

        let port = container.get_host_port_ipv4(27017);
        let connection_string = format!("{}{}", C_STRING, port);

        let insert_result =
            create_user_to_db_with_config(
                &mut User::create_test_user(None),
                &connection_string.clone(),
                DB_NAME
            ).await
        ;

        assert!(insert_result.is_ok());
        let inserted_id = insert_result.unwrap();

        let search_result =
            get_user_from_db_with_config(
                &inserted_id,
                &connection_string,
                DB_NAME
            ).await
        ;

        assert!(search_result.is_ok());
        let user_id = search_result.unwrap().id.unwrap();

        assert_eq!(&inserted_id, &user_id);

        let delete_result =
            remove_user_from_db_with_config(
                &user_id,
                &connection_string,
                DB_NAME
            ).await
        ;

        assert!(delete_result.is_ok());
        assert_eq!(
            get_user_from_db_with_config(
                &user_id,
                &connection_string,
                DB_NAME
            ).await,
            Err(DatabaseError::UserNotFound(user_id))
        );

        container.stop();
    }

    #[tokio::test]
    async fn test_add_and_update() {
        let client = Cli::default();
        let container = client.run(get_mongo_image());

        let port = container.get_host_port_ipv4(27017);
        let connection_string = format!("{}{}", C_STRING, port);

        let insert_result =
            create_user_to_db_with_config(
                &mut User::create_test_user(None),
                &connection_string,
                DB_NAME
            ).await
        ;

        assert!(insert_result.is_ok());
        let inserted_id = insert_result.unwrap();

        let mut user = User::create_test_user(Some(inserted_id.clone()));
        user.name = "NEW NAME".to_string();

        let update_result =
            update_user_in_db_with_config(
                user,
                &connection_string,
                DB_NAME
            ).await
        ;

        assert!(update_result.is_ok());

        let find_result =
            get_user_from_db_with_config(
                inserted_id.as_str(),
                &connection_string,
                DB_NAME
            ).await
        ;

        assert!(find_result.is_ok());
        assert_eq!("NEW NAME".to_string(), find_result.unwrap().name);
    }

    fn get_mongo_image() -> GenericImage {
        GenericImage::new("mongo", "latest")
            .with_env_var("MONGO_INITDB_DATABASE", "showcase_test")
            .with_exposed_port(27017)
    }
}
