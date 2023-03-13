use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Validate, Deserialize, ToSchema, TS)]
pub struct CreateUser {
    #[validate(length(min = 5, max = 60))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Validate, Deserialize, ToSchema, TS)]
pub struct UserLogin {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
pub struct UserClaims {
    pub user: UserResponse,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
pub struct UserToken {
    pub access_token: String,
    pub r#type: String,
}
