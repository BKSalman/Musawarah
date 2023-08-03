use chrono::DateTime;
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::sessions;

#[derive(Queryable, Selectable)]
#[diesel(belongs_to(User))]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub expires_at: DateTime<chrono::Utc>,
    pub user_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CreateSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<chrono::Utc>,
    pub expires_at: DateTime<chrono::Utc>,
}
