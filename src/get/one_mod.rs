use crate::{get::ApiResponse, prelude::*};
use mongodb::bson::doc;
use rocket::{http::Status, State};

#[rocket::get("/<owner>/<name>")]
pub(crate) async fn one_mod(owner: &str, name: &str, mods: &State<Mods>) -> ApiResponse {
    let full_name = format!("{owner}/{name}");
    let query = doc! { "_id": full_name };
    let mongo_mod = match mods.find_one(query, None).await {
        Ok(Some(m)) => m,
        Ok(None) => return Err(Status::NotFound),
        Err(_) => return Err(Status::BadRequest),
    };

    Ok(super::expose_as_json(&mongo_mod))
}
