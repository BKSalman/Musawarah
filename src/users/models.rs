use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Deserialize)]
pub struct CreateUser {
    #[validate(length(min = 5, max = 60))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Validate, Deserialize)]
pub struct UserLogin {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserReponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserClaims {
    user: UserReponse,
}
