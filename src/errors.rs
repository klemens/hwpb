error_chain!{
    foreign_links {
        Csv(::csv::Error);
        Db(::diesel::result::Error);
        Json(::serde_json::error::Error);
    }
}

#[derive(Debug)]
pub enum ApiError {
    /// 422 Unprocessable Entity
    ConstraintViolation,
    /// 423 Locked
    Locked,
    /// 500 Internal Server Error (default)
    Other(Error),
}

pub type ApiResult<T> = ::std::result::Result<T, ApiError>;

impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        ApiError::Other(e)
    }
}

impl From<::csv::Error> for ApiError {
    fn from(e: ::csv::Error) -> Self {
        ApiError::Other(e.into())
    }
}

impl From<::diesel::result::Error> for ApiError {
    fn from(e: ::diesel::result::Error) -> Self {
        use diesel::result::DatabaseErrorKind::ForeignKeyViolation;
        use diesel::result::Error::DatabaseError;

        match e {
            DatabaseError(ForeignKeyViolation, _) => ApiError::ConstraintViolation,
            _ => ApiError::Other(e.into()),
        }
    }
}

use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Response, Responder};

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        match self {
            ApiError::ConstraintViolation => Response::build().status(Status::UnprocessableEntity).ok(),
            ApiError::Locked => Response::build().status(Status::Locked).ok(),
            ApiError::Other(e) => Err::<(), _>(e).respond_to(req), // generic error responder
        }
    }
}
