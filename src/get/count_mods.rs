use crate::prelude::*;
use mongodb::bson::doc;
use rocket::{http::Status, serde::json::Value, State};

#[rocket::get("/count")]
pub(crate) async fn count_mods(mods: &State<Mods>) -> Result<Value, Status> {
    let count = sc!(
        mods.count_documents(doc! {}, None).await,
        Status::InternalServerError
    );

    Ok(count.into())
}
