use axum::{async_trait, http::StatusCode, response::IntoResponse, Json};
use axum_login::{secrecy::SecretVec, AuthUser, UserStore};

use migration::DbErr;
use sea_orm::{DatabaseConnection, EntityTrait};
use uuid::Uuid;

use crate::{entity, ErrorHandlingResponse};

pub type AuthContext =
    axum_login::extractors::AuthContext<Uuid, entity::users::Model, SeaORMUserStore>;

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("invalid session")]
    InvalidSession,

    #[error("something went wrong")]
    InternalServerError(#[from] DbErr),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("auth error: {}", self);
        let (status, error_message) = match self {
            AuthError::InvalidSession => (
                StatusCode::UNAUTHORIZED,
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
            AuthError::InternalServerError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorHandlingResponse {
                    errors: vec![self.to_string()],
                },
            ),
        };

        let body = Json(error_message);

        (status, body).into_response()
    }
}

impl<Role> AuthUser<Uuid, Role> for entity::users::Model
where
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
{
    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_password_hash(&self) -> axum_login::secrecy::SecretVec<u8> {
        SecretVec::new(self.password.clone().into())
    }
}

#[derive(Debug, Clone)]
pub struct SeaORMUserStore {
    conn: DatabaseConnection,
}

impl SeaORMUserStore {
    pub fn new(conn: &DatabaseConnection) -> Self {
        Self { conn: conn.clone() }
    }
}

#[async_trait]
impl<Role> UserStore<Uuid, Role> for SeaORMUserStore
where
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
{
    type User = entity::users::Model;

    async fn load_user(&self, user_id: &Uuid) -> Result<Option<Self::User>, eyre::Report> {
        let user = entity::users::Entity::find_by_id(*user_id)
            .one(&self.conn)
            .await?;
        match user {
            Some(u) => Ok(Some(u)),
            None => Ok(None),
        }
    }
}
