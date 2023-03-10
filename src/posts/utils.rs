use std::error::Error;

use super::PostsError;

pub fn box_error(err: PostsError) -> Box<dyn Error + Sync + Send + 'static> {
    Box::new(err)
}
