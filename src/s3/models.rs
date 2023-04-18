use uuid::Uuid;

#[allow(dead_code)]
pub struct Image {
    id: Uuid,
    user_id: Uuid,
    post_id: Uuid,
    path: String,
}
