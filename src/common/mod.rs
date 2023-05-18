pub mod models;

// TODO: add common error type

// #[derive(thiserror::Error, Debug)]
// pub enum CommonErrors {
//     #[error("internal server error")]
//     InternalServerError,
// }

// impl IntoResponse for CommonErrors {
//     fn into_response(self) -> axum::response::Response {
//         let (status, error_message) = match self {
//             CommonErrors::InternalServerError => (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 ErrorResponse {
//                     errors: vec![self.to_string()],
//                 },
//             ),
//         };

//         let body = Json(error_message);

//         (status, body).into_response()
//     }
// }
