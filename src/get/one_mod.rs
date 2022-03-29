use crate::{get::ApiResponse, prelude::*};
use mongodb::bson::doc;
use rocket::{http::Status, State};

#[rocket::get("/<owner>/<name>")]
pub(crate) async fn one_mod(owner: &str, name: &str, mods: &State<Mods>) -> ApiResponse {
    let full_name = format!("{owner}/{name}");
    let q = doc! { "_id": full_name };
    let entry = sc!(mods.find_one(q, None).await, Status::InternalServerError);

    match entry {
        Some(m) => Ok(super::expose_as_json(&m)),
        None => Err(Status::NotFound),
    }
}
