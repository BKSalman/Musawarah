use crate::Rating;

pub fn average_rating<T: Rating>(ratings: Vec<T>) -> f64 {
    if ratings.is_empty() {
        0.0
    } else {
        ratings.iter().map(|r| r.rating()).sum::<f64>() / ratings.len() as f64
    }
}
