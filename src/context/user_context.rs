use mongodb::bson::oid::ObjectId;


#[derive(Clone, Debug)]
pub struct UserContext{
    pub user_id: ObjectId,
    pub email: String,
}

impl UserContext {
    pub fn new (user_id: ObjectId, email: String) -> Self{
        Self{
            user_id,
            email
        }
    }
}

impl UserContext {
    pub fn user_id(&self) -> ObjectId{
        self.user_id
    }
}
