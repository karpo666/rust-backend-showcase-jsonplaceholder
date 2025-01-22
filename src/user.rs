use std::collections::HashMap;
use serde::{de, Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, PartialEq, Debug, Eq, Clone)]
pub struct User {

    #[serde(deserialize_with = "deserialize_id")]
    pub id: Option<String>,
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
    pub fn _create_test_user(id: Option<String>) -> User {
        User {
            id,
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

/// Custom deserializer for User.id.
fn deserialize_id<'de, D>(de: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>
{

    match Value::deserialize(de)? {
        Value::Number(num) => Ok(Some(num.to_string())),
        Value::String(string) => Ok(Some(string)),
        Value::Null => Ok(None),
        _ => Err(de::Error::custom("Invalid type"))
    }
}