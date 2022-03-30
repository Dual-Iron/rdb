use mongodb::options::UpdateOptions;
use rocket::{http::Status, response::status::Custom};

pub use crate::guards::*;
pub use crate::serde::Verification::*;
pub use crate::serde::*;
pub use mongodb::bson::{doc, to_bson};

pub(crate) type Mods = mongodb::Collection<crate::serde::ModEntry>;
pub(crate) type SimpleResponse = Result<&'static str, Custom<&'static str>>;

pub(crate) fn client_err<R>(message: R) -> Custom<R> {
    Custom(Status::BadRequest, message)
}

pub(crate) fn server_err<R>(message: R) -> Custom<R> {
    Custom(Status::InternalServerError, message)
}

// Shorthand for 'upsert' options
pub(crate) fn upsert() -> UpdateOptions {
    UpdateOptions::builder().upsert(true).build()
}

// Shorthand for short-circuiting bad results
macro_rules! sc {
    ($e:expr, $err:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => {
                dbg!(e);
                return Err($err);
            }
        }
    }};
}

pub(crate) use sc;
