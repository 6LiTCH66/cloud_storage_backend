use std::env;
use dotenv::dotenv;
use mongodb::bson::oid::ObjectId;
use mongodb::{Client, Collection};
use mongodb::options::ClientOptions;
use mongodb::results::InsertOneResult;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct User{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password: String,
}

