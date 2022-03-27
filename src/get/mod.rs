use crate::prelude::*;
use rocket::serde::json::{serde_json::json, Value};

pub(crate) mod index;
pub(crate) mod many_mods;
pub(crate) mod one_mod;

type ApiResponse = Result<Value, rocket::http::Status>;

fn expose_as_json(entry: &ModEntry) -> Value {
    let (owner, name) = entry.id.split_once('/').unwrap_or(("no-name", &entry.id));
    json!({
        "name": name,
        "owner": owner,
        "updated": entry.updated,
        "downloads": entry.downloads.unwrap_or(0),
        "description": &entry.info.description,
        "homepage": &entry.info.homepage,
        "version": &entry.info.version,
        "binaries": &entry.info.binaries
    })
}
