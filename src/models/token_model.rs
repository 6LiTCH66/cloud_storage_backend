use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: Option<ObjectId>,
    pub email: String,
    pub exp: i64,
}