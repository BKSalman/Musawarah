use chrono::DateTime;
use diesel::{Identifiable, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::email_verifications;

#[derive(Queryable, Selectable, Insertable, Debug, Identifiable)]
#[diesel(belongs_to(User))]
#[diesel(table_name = email_verifications)]
pub struct EmailVerification {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<chrono::Utc>,
    pub expires_at: DateTime<chrono::Utc>,
    pub user_id: Uuid,
}

// #[derive(Deserialize, Serialize, Debug, ToSchema, TS)]
// #[ts(export)]
// pub struct CreateEmailVerification {
//     pub user_id: Uuid,
// }
