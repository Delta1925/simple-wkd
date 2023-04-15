use actix_web::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("(0x01) Cert is invalid")]
    InvalidCert,
    #[error("(0x02) Error while parsing cert")]
    ParseCert,
    #[error("(0x03) Error while parsing an E-Mail address")]
    ParseEmail,
    #[error("(0x04) There is no pending request associated to this token")]
    MissingPending,
    #[error("(0x05) Requested key does not exist")]
    MissingKey,
    #[error("(0x06) No E-Mail found in the certificate")]
    MissingMail,
    #[error("(0x07) Error while sending the E-Mail")]
    SendMail,
    #[error("(0x08) rror while serializing data")]
    SerializeData,
    #[error("(0x09) Error while deserializing data")]
    DeserializeData,
    #[error("(0x0A) The file is inaccessible")]
    Inaccessible,
    #[error("(0x0B) Error while adding a key to the wkd")]
    AddingKey,
    #[error("(0x0C) Error while generating the wkd path")]
    PathGeneration,
    #[error("(0x0D) Error while generating the email")]
    MailGeneration,
    #[error("(0x0E) Wrong email domain")]
    WrongDomain,
    #[error("(0x0F) The requested file does not exist")]
    MissingFile,
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::MissingPending => StatusCode::from_u16(404).unwrap(),
            Self::MissingKey => StatusCode::from_u16(404).unwrap(),
            Self::MissingFile => StatusCode::from_u16(404).unwrap(),
            Self::WrongDomain => StatusCode::from_u16(401).unwrap(),
            _ => StatusCode::from_u16(500).unwrap(),
        }
    }
}
