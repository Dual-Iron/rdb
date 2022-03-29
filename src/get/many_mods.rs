use crate::{get::ApiResponse, prelude::*};
use mongodb::{bson::doc, options::FindOptions};
use rocket::{
    http::Status,
    serde::json::{serde_json::json, Value},
};

#[rocket::get("/?<page>&<sort>&<search>")]
pub(crate) async fn many_mods(
    page: Option<u64>,
    sort: Option<&str>,
    search: Option<&str>,
    mods: &rocket::State<Mods>,
) -> ApiResponse {
    use rocket::futures::TryStreamExt;

    let sort = get_sort(sort.unwrap_or("new")).ok_or(Status::BadRequest)?;
    let query = get_search(search);
    let options = Some(
        FindOptions::builder()
            .sort(Some(sort))
            .skip(page.unwrap_or(0))
            .limit(20)
            .build(),
    );

    let cursor = sc!(mods.find(query, options).await, Status::InternalServerError);
    let mods: Vec<ModEntry> = sc!(cursor.try_collect().await, Status::InternalServerError);
    let mods_json: Vec<Value> = mods.iter().map(super::expose_as_json).collect();

    Ok(json!(mods_json))
}

fn get_search(search: Option<&str>) -> mongodb::bson::Document {
    match search {
        Some(s) => doc! {
            "$text": {
                "$search": s,
                "$caseSensitive": false,
                "$diacriticSensitive": false
            }
        },
        None => doc! {},
    }
}

fn get_sort(sort: &str) -> Option<mongodb::bson::Document> {
    match sort {
        "new" => Some(doc! { "updated": -1 }),
        "old" => Some(doc! { "updated": 1 }),
        "most-downloads" => Some(doc! { "downloads": -1  }),
        "least-downloads" => Some(doc! { "downloads": 1 }),
        _ => None,
    }
}
