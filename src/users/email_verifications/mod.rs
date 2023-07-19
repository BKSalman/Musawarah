use axum::{http::StatusCode, response::IntoResponse};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};

use crate::{ErrorResponse, EMAIL_PASSWORD, EMAIL_SMTP_SERVER, EMAIL_USERNAME};

use self::models::EmailVerification;

pub mod models;
pub mod routes;

#[derive(thiserror::Error, Debug)]
pub enum EmailVerificationError {
    #[error(transparent)]
    PoolError(#[from] diesel_async::pooled_connection::deadpool::PoolError),

    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),

    #[error("Email has expired")]
    ExpiredEmail,

    #[error(transparent)]
    EmailSendError(#[from] lettre::transport::smtp::Error),

    #[error(transparent)]
    BodyCreationError(#[from] lettre::error::Error),
}

impl EmailVerification {
    async fn send_email(&self, username: String) -> Result<(), EmailVerificationError> {
        let email_username = EMAIL_USERNAME.get().expect("Email Username");
        let email_password = EMAIL_PASSWORD.get().expect("Email Password");
        let email_smtp_server = EMAIL_SMTP_SERVER.get().expect("Email SMTP Server");

        let from = format!("Musawarah <{}>", email_username);
        let to = format!("{} <{}>", username, self.email);

        // TODO: add pretty html to the email
        let email = Message::builder()
            .from(from.parse().expect("Valid SMTP from field"))
            .to(to.parse().expect("Valid SMTP to field"))
            .subject("Verify Musawarah Account!")
            .header(ContentType::TEXT_PLAIN)
            .body(format!(
                "Click below to verify your account.\nhttp://localhost:5173/confirm-email/{}",
                self.id
            ))?;

        let creds = Credentials::new(email_username.clone(), email_password.clone());

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(email_smtp_server)?
                .credentials(creds)
                .build();

        mailer.send(email).await?;
        Ok(())
    }
}

impl IntoResponse for EmailVerificationError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:#?}", self);

        match self {
            Self::Diesel(diesel_error) => {
                if let diesel::result::Error::NotFound = diesel_error {
                    return (
                        StatusCode::NOT_FOUND,
                        ErrorResponse {
                            error: String::from("verification id not found"),
                            ..Default::default()
                        },
                    )
                        .into_response();
                }
                (StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
            Self::PoolError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            Self::EmailSendError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            Self::BodyCreationError(_) => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
            Self::ExpiredEmail => (
                StatusCode::GONE,
                ErrorResponse {
                    error: self.to_string(),
                    ..Default::default()
                },
            )
                .into_response(),
        }
    }
}
