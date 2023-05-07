use async_trait::async_trait;
use axum_login::axum_sessions::async_session::{Result, Session, SessionStore};
use chrono::{Duration, Utc};
use migration::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter,
};

use crate::entity;

#[derive(Clone, Debug)]
pub struct SeaORMSessionStore {
    connection: DatabaseConnection,
}

impl SeaORMSessionStore {
    pub fn new(connection: &DatabaseConnection) -> Self {
        Self {
            connection: connection.clone(),
        }
    }
}

#[async_trait]
impl SessionStore for SeaORMSessionStore {
    async fn load_session(&self, cookie_value: String) -> Result<Option<Session>> {
        let id = Session::id_from_cookie_value(&cookie_value)?;

        let session = entity::sessions::Entity::find_by_id(id)
            .filter(entity::sessions::Column::ExpiresAt.gt(Utc::now().naive_utc()))
            .one(&self.connection)
            .await?;

        Ok(session
            .map(|session| serde_json::from_str(&session.session))
            .transpose()?)
    }

    async fn store_session(&self, session: Session) -> Result<Option<String>> {
        let id = session.id();
        let string = serde_json::to_string(&session)?;

        let created_at = Utc::now();
        let expires_at = created_at + Duration::hours(2);

        let _res = entity::sessions::Entity::insert(
            entity::sessions::Model {
                id: id.to_string(),
                created_at: created_at.naive_utc(),
                expires_at: expires_at.naive_utc(),
                session: string,
            }
            .into_active_model(),
        )
        .on_conflict(
            // on conflict overwrite old expires_at, created_at, session
            OnConflict::column(entity::sessions::Column::Id)
                .update_columns([
                    entity::sessions::Column::ExpiresAt,
                    entity::sessions::Column::CreatedAt,
                    entity::sessions::Column::Session,
                ])
                .to_owned(),
        )
        .exec_with_returning(&self.connection)
        .await?;

        Ok(session.into_cookie_value())
    }

    async fn destroy_session(&self, session: Session) -> Result {
        let id = session.id();

        let _deleted = entity::sessions::ActiveModel {
            id: ActiveValue::set(id.to_string()),
            ..Default::default()
        }
        .delete(&self.connection)
        .await?;

        Ok(())
    }

    async fn clear_store(&self) -> Result {
        entity::sessions::Entity::delete_many()
            .exec(&self.connection)
            .await?;

        Ok(())
    }
}
