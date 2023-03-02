use serde::{Deserialize, Serialize};

// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
pub struct UserReponse {
    pub id: String,
    pub username: String,
}
