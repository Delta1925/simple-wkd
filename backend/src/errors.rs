use actix_web::{http::StatusCode, HttpResponseBuilder, ResponseError};
use anyhow::Error;
use std::fmt::Display;
use thiserror::Error as DeriveError;

use crate::utils::return_outcome;

#[macro_export]
macro_rules! log_err {
    ($var: expr, $level: ident) => {{
        let test = $var;
        if test.is_err() {
            $level!(
                "{} {}",
                $crate::settings::ERROR_TEXT,
                test.as_ref().unwrap_err()
            );
            test
        } else {
            test
        }
    }};
    ($var: expr, $level: ident, $panic: expr) => {{
        let test = $var;
        if log_err!(test, $level).is_err() {
            if $panic == true {
                panic!("{} {}", $crate::settings::ERROR_TEXT, test.unwrap_err());
            } else {
                test
            }
        } else {
            $var
        }
    }};
}

#[derive(Debug, DeriveError)]
pub enum SpecialErrors {
    #[error("Could not find any primay user email in the keyblock!")]
    EmailMissing,
    #[error("The request had expired!")]
    ExpiredRequest,
    #[error("The key for the requested user does not exist!")]
    InexistingUser,
    #[error("The key is either expired or uses an obsolete cipher!")]
    InvalidCert,
    #[error("Could not parse keyblock")]
    MalformedCert,
    #[error("Could not parse user email: malformed email")]
    MalformedEmail,
    #[error("The requested file does not exist!")]
    MissingFile,
    #[error("User email rejected: domain not allowed")]
    UnallowedDomain,
}

#[derive(Debug)]
pub enum CompatErr {
    AnyhowErr(Error),
    SpecialErr(SpecialErrors),
}

impl Display for CompatErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnyhowErr(error) => write!(f, "{}", error),
            Self::SpecialErr(error) => write!(f, "{}", error),
        }
    }
}

impl From<SpecialErrors> for CompatErr {
    fn from(value: SpecialErrors) -> Self {
        CompatErr::SpecialErr(value)
    }
}

impl From<Error> for CompatErr {
    fn from(value: Error) -> Self {
        if value.is::<SpecialErrors>() {
            CompatErr::from(value.downcast::<SpecialErrors>().unwrap())
        } else {
            CompatErr::AnyhowErr(value)
        }
    }
}

impl ResponseError for CompatErr {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::AnyhowErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SpecialErr(error) => match error {
                SpecialErrors::ExpiredRequest => StatusCode::BAD_REQUEST,
                SpecialErrors::InexistingUser => StatusCode::NOT_FOUND,
                SpecialErrors::InvalidCert => StatusCode::BAD_REQUEST,
                SpecialErrors::EmailMissing => StatusCode::BAD_REQUEST,
                SpecialErrors::MalformedCert => StatusCode::BAD_REQUEST,
                SpecialErrors::MalformedEmail => StatusCode::BAD_REQUEST,
                SpecialErrors::MissingFile => StatusCode::NOT_FOUND,
                SpecialErrors::UnallowedDomain => StatusCode::UNAUTHORIZED,
            },
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match return_outcome(Err(self)) {
            Ok(httpbuilder) => httpbuilder,
            Err(_) => HttpResponseBuilder::new(self.status_code()).body(self.to_string()),
        }
    }
}
