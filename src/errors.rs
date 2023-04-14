use actix_web::http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug, Clone, Copy)]
pub enum Error {
    #[error("EP1: Error while parsing cert")]
    ParseCert,
    #[error("EP2: Error while parsing an E-Mail address")]
    ParseEmail,
    #[error("EM1: There is no pending request associated to this token")]
    MissingPending,
    #[error("EM2: Requested key does not exist")]
    MissingKey,
    #[error("EM3: No E-Mail found in the certificate")]
    MissingMail,
    #[error("EE1: Error while sending the E-Mail")]
    SendMail,
    #[error("EE2: Error while building the SMTP connection")]
    SmtpBuilder,
    #[error("ES1: rror while serializing data")]
    SerializeData,
    #[error("ES2: Error while deserializing data")]
    DeserializeData,
    #[error("ES3: The file is inaccessible")]
    Inaccessible,
    #[error("ES4: Error while adding a key to the wkd")]
    AddingKey,
    #[error("EG1: Error while generating the wkd path")]
    PathGeneration,
    #[error("EG2: Error while generating the email")]
    MailGeneration,
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
