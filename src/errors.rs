use actix_web::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("Error while parsing cert")]
    ParseCert,
    #[error("Error while parsing an E-Mail address")]
    ParseEmail,
    #[error("There is no pending request associated to this token")]
    MissingPending,
    #[error("Requested key does not exist")]
    MissingKey,
    #[error("No E-Mail found in the certificate")]
    MissingMail,
    #[error("Error while serializing data")]
    SerializeData,
    #[error("Error while deserializing data")]
    DeserializeData,
    #[error("The file is inaccessible")]
    Inaccessible,
    #[error("Error while adding a key to the wkd")]
    AddingKey,
    #[error("Error while generating the wkd path")]
    PathGeneration,
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::MissingPending => StatusCode::from_u16(404).unwrap(),
            Self::MissingKey => StatusCode::from_u16(404).unwrap(),
            _ => StatusCode::from_u16(500).unwrap(),
        }
    }
}
