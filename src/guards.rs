use rocket::{
    request::{FromRequest, Outcome},
    Request,
};

macro_rules! event_type_guard {
    ($type_name:ident, $header_name:literal) => {
        pub struct $type_name;

        #[rocket::async_trait]
        impl<'r> FromRequest<'r> for $type_name {
            type Error = ();

            async fn from_request(req: &'r Request<'_>) -> Outcome<Self, ()> {
                match req.headers().get_one("X-GitHub-Event") {
                    Some($header_name) => Outcome::Success($type_name),
                    _ => Outcome::Forward(()),
                }
            }
        }
    };
}

event_type_guard!(RelGuard, "release");
event_type_guard!(PingGuard, "ping");
