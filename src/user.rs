use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Clone)]
pub struct User {

    #[serde(rename = "id")]
    pub user_id: Option<i32>,
    pub name: String,
    pub username: String,
    pub email: String,
    pub address: Address,
    pub phone: String,
    pub website: String,
    pub company: Company,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Address {
    pub street: String,
    pub suite: String,
    pub city: String,
    pub geo: HashMap<String, String>
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Company {
    pub name: String,
    #[serde(rename = "catchPhrase")]
    pub catch_phrase: String,
    pub bs: String
}

impl User {

    /// Create a new user. Meant for testing.
    pub fn create_test_user(id: i32) -> User {
        User {
            user_id: Some(id),
            name: "TESTER".to_string(),
            username: "TESTER_69".to_string(),
            email: "testlover@testing.gov".to_string(),
            address: Address {
                street: "Totallyrealstreet 6".to_string(),
                suite: "a 12".to_string(),
                city: "Testington".to_string(),
                geo: [
                    ("lat".to_string(), "12".to_string()),
                    ("lon".to_string(), "15".to_string())
                ].iter().cloned().collect(),
            },
            phone: "123456789".to_string(),
            website: "testing.gov".to_string(),
            company: Company {
                name: "Testing".to_string(),
                catch_phrase: "Truly we are testing".to_string(),
                bs: "To test".to_string(),
            },
        }
    }
}
