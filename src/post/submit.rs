use crate::prelude::*;
use rocket::{post, serde::json::Json, State};

#[post("/", data = "<data>", format = "application/json")]
pub(crate) async fn submit(data: Json<Submission>, mods: &State<Mods>) -> SimpleResponse {
    let entry = ModEntry::from_submission(data.0).map_err(client_err)?;

    match entry.verify(mods).await {
        Success | NotFound => insert_mod(&entry, mods).await,
        Old => Err(client_err("The version was outdated.")),
        Failure => Err(client_err("The secret was incorrect.")),
        Error(e) => {
            dbg!(e);
            Err(server_err("There was an internal error."))
        }
    }
}

async fn insert_mod(entry: &ModEntry, mods: &Mods) -> SimpleResponse {
    let query = doc! { "_id": &entry.id };
    let update = doc! {
        "$setOnInsert": {
            "_id": &entry.id,
            "downloads": 0u32,
            "secret": &entry.secret,
            "search": &entry.search,
            "published": entry.published
        },
        "$set": {
            "updated": entry.updated,
            "info": to_bson(&entry.info).expect("Failed to deser ModInfo")
        }
    };

    match mods.update_one(query, update, upsert()).await {
        Ok(r) => match r.upserted_id.is_some() {
            true => Ok("Successfully inserted mod."),
            false => Ok("Successfully updated mod."),
        },
        Err(e) => {
            dbg!(e);
            Err(server_err(
                "Failed to upsert mod because of an internal error.",
            ))
        }
    }
}
