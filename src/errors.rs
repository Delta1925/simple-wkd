use actix_web::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error while parsing cert")]
    ParseCert,
    #[error("Error while parsing E-Mail")]
    ParseMail,
    #[error("Error while parsing stored data")]
    ParseStored,
    #[error("No E-Mail found in the certificate")]
    MissingMail,
    #[error("The E-Mail is malformed")]
    MalformedMail,
    #[error("Error while serializing data")]
    SerializeData,
    #[error("Error while deserializing data")]
    DeserializeData,
    #[error("File or directory does not exist")]
    MissingPath,
    #[error("Requested key does not exist")]
    MissingKey,
    #[error("The file is inaccessible")]
    Inaccessible,
    #[error("Error while adding a key to the wkd")]
    AddingKey,
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::MissingPath => StatusCode::from_u16(404).unwrap(),
            Self::MissingKey => StatusCode::from_u16(404).unwrap(),
            _ => StatusCode::from_u16(500).unwrap(),
        }
    }
}
