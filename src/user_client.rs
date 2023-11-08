use reqwest::header::{ACCEPT, CONTENT_TYPE};
use reqwest::StatusCode;
use url::Url;
use crate::CONFIG;
use crate::user::User;

/// Possible errors thrown by `user_client` functions.
#[derive(Eq, PartialEq, Debug)]
pub enum UserClientError {
    UserNotFound(String),
    RestError(StatusCode),
    UrlParseError,
    SerdeError,
    NoIdError
}

const PATH: &str = "/users";

/// Fetch all users.
pub async fn get_users() -> Result<Vec<User>, UserClientError> {
    get_users_with_url(&CONFIG.json_placeholder.url).await
}

/// Fetch all users from the given url.
///
/// ## Arguments.
/// * `url` - Url where users should be fetched. "/users" will be added to the end of base url.
async fn get_users_with_url(url: &str) -> Result<Vec<User>, UserClientError> {

    // Parse url and handle possible error.
    let url_result =
        Url::parse(url).and_then(
            |url|  url.join(PATH)
        )
    ;
    let url = url_result.map_err(|_| UserClientError::UrlParseError)?;

    // Create client.
    let client = reqwest::Client::new();

    // Create request and send it.
    let response = client.get(url)
        .header(ACCEPT, "application/json")
        .send()
        .await
    ;

    // Check for errors and status-codes other than 200 - OK.
    if let Err(e) = response {
        return Err(UserClientError::RestError(e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)))
    };

    let response = response.unwrap();

    match response.status() {
        StatusCode::OK => (),
        _ => return Err(UserClientError::RestError(response.status()))
    };


    let response_text = response.text().await;
    // Deserialize and return.
    match serde_json::from_str(response_text.unwrap().as_str()) {
        Ok(user) => Ok(user),
        Err(e) => {
            println!("{}", e);
            Err(UserClientError::SerdeError)
        }
    }
}

/// Get user with a specific id.
///
/// ## Arguments.
/// * `id` - Id for the user to be fetched.
pub async fn get_user(id: String) -> Result<User, UserClientError> {
    get_user_with_url(id, &CONFIG.json_placeholder.url).await
}

/// Get user with a specific id.
///
/// ## Arguments.
/// * `id` - Id for the user to be fetched.
/// * `url` - Url where users should be fetched. "/users" and the id will be added to the end of base url.
async fn get_user_with_url(id: String, url: &str) -> Result<User, UserClientError> {

    // Parse url and handle possible errors.
    let url_result =
        Url::parse(url).and_then(
            |url|  url.join(format!("{}/{}", PATH, id).as_str())
        )
    ;
    let url = url_result.map_err(|_| UserClientError::UrlParseError)?;

    // Create client.
    let client = reqwest::Client::new();

    // Create request and send it.
    let response = client.get(url)
        .header(ACCEPT, "application/json")
        .send()
        .await
    ;

    // Check for errors and status-codes other than 200 - OK.
    if let Err(e) = response {
        return Err(UserClientError::RestError(e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)))
    };

    let response = response.unwrap();

    match response.status() {
        StatusCode::OK => (),
        StatusCode::NOT_FOUND => return Err(UserClientError::UserNotFound(id)),
        _ => return Err(UserClientError::RestError(response.status()))
    };


    // Deserialize and return.
    match serde_json::from_str(response.text().await.unwrap().as_str()) {
        Ok(user) => Ok(user),
        Err(_) => Err(UserClientError::SerdeError)
    }
}

/// Post a new user.
///
/// ## Arguments.
/// * `user` - New user info.
async fn post_new_user(user: User) -> Result<User, UserClientError> {
    post_new_user_with_url(user, &CONFIG.json_placeholder.url).await
}

/// Post a new user.
///
/// ## Arguments.
/// * `user` - New user info.
/// * `url` - Url where user should be posted. "/users" will be added to the end of base url.
async fn post_new_user_with_url(user: User, url: &str) -> Result<User, UserClientError> {

    // Parse url and handle possible errors.
    let url_result = Url::parse(url).and_then(
        |url| url.join(PATH)
    );

    let url = url_result.map_err(|_| UserClientError::UrlParseError)?;

    // Create client.
    let client = reqwest::Client::new();

    // Create request and send it.
    let response = client.post(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(serde_json::to_string(&user).map_err(|_| UserClientError::SerdeError)?)
        .send()
        .await
    ;

    // Handle possible errors and status codes other than 200 - OK.
    if let Err(e) = response {
        return Err(UserClientError::RestError(e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)))
    }

    let response = response.unwrap();

    match response.status() {
        StatusCode::OK => (),
        _ => return Err(UserClientError::RestError(response.status()))
    }

    // Deserialize and return.
    match serde_json::from_str(response.text().await.unwrap().as_str()) {
        Ok(user) => Ok(user),
        Err(e) => {
            println!("{}", e);
            Err(UserClientError::SerdeError)
        }
    }
}

/// Update an existing user info.
///
/// ## Arguments.
/// * `user` - Updated user info.
pub async fn update_existing_user(user: User) -> Result<User, UserClientError> {
    if let None = &user.id {
        return Err(UserClientError::NoIdError);
    };
    update_existing_user_with_url(user, &CONFIG.json_placeholder.url).await
}

/// Update an existing user info.
///
/// ## Arguments.
/// * `user` - Updated user info.
/// * `url` - Url where users should be fetched. "/users" will be added to the end of base url.
async fn update_existing_user_with_url(user: User, url: &str) -> Result<User, UserClientError> {

    // Parse url and handle possible errors.
    let url_result = Url::parse(url).and_then(
        |url| url.join(PATH)
    );

    let url = url_result.map_err(|_| UserClientError::UrlParseError)?;

    // Create client.
    let client = reqwest::Client::new();

    // Create request and send it.
    let response = client.patch(url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .body(serde_json::to_string(&user).map_err(|_| UserClientError::SerdeError)?)
        .send()
        .await
    ;

    // Handle possible errors and status codes other than 200 - OK.
    if let Err(e) = response {
        return Err(UserClientError::RestError(e.status().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)))
    }

    let response = response.unwrap();

    match response.status() {
        StatusCode::OK => (),
        StatusCode::NOT_FOUND => return Err(UserClientError::UserNotFound(user.id.unwrap().to_string())),
        _ => return Err(UserClientError::RestError(response.status()))
    }

    // Deserialize and return.
    match serde_json::from_str(response.text().await.unwrap().as_str()) {
        Ok(user) => Ok(user),
        Err(_) => Err(UserClientError::SerdeError)
    }
}

#[cfg(test)]
mod test {
    use httpmock::Method::{GET, PATCH, POST};
    use reqwest::header::CONTENT_TYPE;
    use serde_json::json;
    use super::*;
    use crate::user::User;

    #[tokio::test]
    async fn test_get_users_faulty_url() {
        assert_eq!(
            Err(UserClientError::UrlParseError),
            get_users_with_url("THIS IS A FAULTY URL").await
        );
    }

    #[tokio::test]
    async fn test_get_users_status_not_ok() {
        let mock_server = httpmock::MockServer::start();

        let get_users_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::BAD_REQUEST.into())
                .header(CONTENT_TYPE.as_str(), "application/json");
        });

        assert_eq!(
            Err(UserClientError::RestError(StatusCode::BAD_REQUEST)),
            get_users_with_url(mock_server.url("").as_str()).await
        );

        get_users_mock.assert();
    }

    #[tokio::test]
    async fn test_get_users_faulty_json() {
        let mock_server = httpmock::MockServer::start();

        let get_users_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::OK.into())
                .header(CONTENT_TYPE.as_str(), "application/json")
                .body("THIS IS JUST NOT JSON MAN...");
        });

        assert_eq!(
            Err(UserClientError::SerdeError),
            get_users_with_url(mock_server.url("").as_str()).await
        );

        get_users_mock.assert();
    }

    #[tokio::test]
    async fn test_get_users_success() {
        let mock_server = httpmock::MockServer::start();

        let get_users_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::OK.into())
                .header(CONTENT_TYPE.as_str(), "application/json")
                .body_from_file("testdata/get_users_response.json");
        });

        let response_result = get_users_with_url(mock_server.url("").as_str()).await;
        assert!(response_result.is_ok());

        let response: Vec<User> = response_result.unwrap();
        assert_eq!(10, response.len());
        assert!(response.get(0).unwrap().id.is_some());
        assert_eq!("1".to_string(), response.get(0).unwrap().id.clone().unwrap());

        for (i, user) in response.iter().enumerate() {
            assert_eq!((i as i32 + 1).to_string(), user.id.clone().unwrap());
        }

        get_users_mock.assert();
    }

    #[tokio::test]
    async fn test_get_user_faulty_url() {
        assert_eq!(
            Err(UserClientError::UrlParseError),
            get_user_with_url(String::from("TEST_ID"), "THIS IS A FAULTY URL").await
        );
    }

    #[tokio::test]
    async fn test_get_user_status_not_ok() {
        let mock_server = httpmock::MockServer::start();

        let get_users_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path_contains("TEST_ID")
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::BAD_REQUEST.into())
                .header(CONTENT_TYPE.as_str(), "application/json");
        });

        assert_eq!(
            Err(UserClientError::RestError(StatusCode::BAD_REQUEST)),
            get_user_with_url("TEST_ID".to_string(), mock_server.url("").as_str()).await
        );

        get_users_mock.assert();
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mock_server = httpmock::MockServer::start();

        let get_user_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .header(ACCEPT.as_str(), "application/json");
            then.status(404)
                .header(CONTENT_TYPE.as_str(), "application/json");
        });

        assert_eq!(
            Err(UserClientError::UserNotFound(String::from("100"))),
            get_user_with_url(String::from("100"), mock_server.url("").as_str()).await
        );

        get_user_mock.assert();
    }

    #[tokio::test]
    async fn test_get_user_faulty_json() {
        let mock_server = httpmock::MockServer::start();

        let get_users_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .path_contains("TEST_ID")
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::OK.into())
                .header(CONTENT_TYPE.as_str(), "application/json")
                .body("THIS IS NOT JSON!!!!");
        });

        assert_eq!(
            Err(UserClientError::SerdeError),
            get_user_with_url("TEST_ID".to_string(), mock_server.url("").as_str()).await
        );

        get_users_mock.assert();
    }

    #[tokio::test]
    async fn test_get_user() {
        let mock_server = httpmock::MockServer::start();

        let get_user_mock = mock_server.mock(|when, then| {
            when.method(GET)
                .header(ACCEPT.as_str(), "application/json")
                .path_contains("TEST_ID");
            then.status(StatusCode::OK.into())
                .header(CONTENT_TYPE.as_str(), "application/json")
                .body_from_file("testdata/get_user_response.json");
        });

        let response_result = get_user_with_url("TEST_ID".to_string(), mock_server.url("").as_str()).await;
        assert!(response_result.is_ok());

        let response: User = response_result.unwrap();
        assert!(response.id.is_some());
        assert_eq!("1".to_string(), response.id.unwrap());
        assert_eq!("Leanne Graham", response.name);

        get_user_mock.assert()
    }

    #[tokio::test]
    async fn test_post_new_user_faulty_url() {
        assert_eq!(
            Err(UserClientError::UrlParseError),
            post_new_user_with_url(User::create_test_user(None), "THIS IS NOT A REAL URL").await
        );
    }

    #[tokio::test]
    async fn test_post_new_user_status_not_ok() {
        let mock_server = httpmock::MockServer::start();

        let post_user_mock = mock_server.mock(|when, then| {
            when.method(POST)
                .header(CONTENT_TYPE.as_str(), "application/json")
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::BAD_REQUEST.into())
                .header(CONTENT_TYPE.as_str(), "application/json");
        });

        assert_eq!(
            Err(UserClientError::RestError(StatusCode::BAD_REQUEST)),
            post_new_user_with_url(User::create_test_user(None), mock_server.url("").as_str()).await
        );

        post_user_mock.assert();
    }

    #[tokio::test]
    async fn post_new_user_faulty_json() {
        let mock_server = httpmock::MockServer::start();

        let post_user_mock = mock_server.mock(|when, then| {
            when.method(POST)
                .header(CONTENT_TYPE.as_str(), "application/json")
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::OK.into())
                .header(CONTENT_TYPE.as_str(), "application/json")
                .body("THIS IS NOT JSON BELIEVE ME");
        });

        assert_eq!(
            Err(UserClientError::SerdeError),
            post_new_user_with_url(User::create_test_user(None), mock_server.url("").as_str()).await
        );

        post_user_mock.assert();
    }

    #[tokio::test]
    async fn test_post_new_user() {
        let mock_server = httpmock::MockServer::start();

        let new_user_info = User::create_test_user(Some(0.to_string()));

        let post_user_mock = mock_server.mock(|when, then| {
            when.method(POST)
                .header(CONTENT_TYPE.as_str(), "application/json")
                .header(ACCEPT.as_str(), "application/json")
                .json_body(json!(User::create_test_user(Some(0.to_string()))));
            then.status(StatusCode::OK.into())
                .header(CONTENT_TYPE.as_str(), "application/json")
                .body_from_file("testdata/get_user_response.json");
        });

        let response_result = post_new_user_with_url(new_user_info.clone(), mock_server.url("").as_str()).await;
        dbg!(&response_result);
        assert!(response_result.is_ok());

        let response = response_result.unwrap();
        assert!(response.id.is_some());
        assert_eq!(1.to_string(), response.id.unwrap());

        post_user_mock.assert();
    }

    #[tokio::test]
    async fn test_update_existing_user_faulty_url() {
        assert_eq!(
            Err(UserClientError::UrlParseError),
            update_existing_user_with_url(User::create_test_user(None), "THIS IS NOT A PROPER URL.").await
        );
    }

    #[tokio::test]
    async fn test_update_existing_user_status_not_ok() {
        let mock_server = httpmock::MockServer::start();

        let update_user_mock = mock_server.mock(|when, then| {
            when.method(PATCH)
                .header(CONTENT_TYPE.as_str(), "application/json")
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::BAD_REQUEST.into())
                .header(CONTENT_TYPE.as_str(), "application/json");
        });

        assert_eq!(
            Err(UserClientError::RestError(StatusCode::BAD_REQUEST)),
            update_existing_user_with_url(User::create_test_user(None), mock_server.url("").as_str()).await
        );

        update_user_mock.assert();
    }

    #[tokio::test]
    async fn test_update_existing_user_not_found() {
        let mock_server = httpmock::MockServer::start();

        let update_user_mock = mock_server.mock(|when, then| {
            when.method(PATCH)
                .header(CONTENT_TYPE.as_str(), "application/json")
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::NOT_FOUND.into())
                .header(CONTENT_TYPE.as_str(), "application/json");
        });

        assert_eq!(
            Err(UserClientError::UserNotFound(0.to_string())),
            update_existing_user_with_url(User::create_test_user(Some(0.to_string())), mock_server.url("").as_str()).await
        );

        update_user_mock.assert();
    }

    #[tokio::test]
    async fn test_update_existing_user_faulty_json() {
        let mock_server = httpmock::MockServer::start();

        let update_user_mock = mock_server.mock(|when, then| {
            when.method(PATCH)
                .header(CONTENT_TYPE.as_str(), "application/json")
                .header(ACCEPT.as_str(), "application/json");
            then.status(StatusCode::OK.into())
                .header(CONTENT_TYPE.as_str(), "application/json")
                .body("THIS IS NOT JSON");
        });

        assert_eq!(
            Err(UserClientError::SerdeError),
            update_existing_user_with_url(User::create_test_user(None), mock_server.url("").as_str()).await
        );

        update_user_mock.assert();
    }

    #[tokio::test]
    async fn test_update_existing_user() {
        let mock_server = httpmock::MockServer::start();

        let user_info_to_be_updated = User::create_test_user(Some(10.to_string()));

        let update_user_mock = mock_server.mock(|when, then| {
            when.method(PATCH)
                .header(CONTENT_TYPE.as_str(), "application/json")
                .header(ACCEPT.as_str(), "application/json")
                .json_body(json!(user_info_to_be_updated.clone()));
            then.status(StatusCode::OK.into())
                .header(CONTENT_TYPE.as_str(), "application/json")
                .body_from_file("testdata/get_user_response.json");
        });

        let response_result = update_existing_user_with_url(user_info_to_be_updated.clone(), mock_server.url("").as_str()).await;
        assert!(response_result.is_ok());

        let response = response_result.unwrap();
        assert!(response.id.is_some());
        assert_eq!(1.to_string(), response.id.unwrap());

        update_user_mock.assert();
    }
}
