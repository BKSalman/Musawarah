use std::error::Error;

use super::ChaptersError;

pub fn box_error(err: ChaptersError) -> Box<dyn Error + Sync + Send + 'static> {
    Box::new(err)
}
