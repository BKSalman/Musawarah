use uuid::Uuid;

pub struct Image {
    id: Uuid,
    user_id: Uuid,
    post_id: Uuid,
    path: String,
}
