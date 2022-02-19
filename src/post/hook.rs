use crate::prelude::*;
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};

#[rocket::post("/", format = "application/json")]
pub(crate) async fn ping(_e: PingGuard) -> &'static str {
    "Successfully connected to rdb! Make sure that:
- You set content type to \"application/json\",
- You enabled Release events, and
- SSL verification is enabled."
}

#[rocket::post("/?<secret>", data = "<data>", format = "application/json")]
pub(crate) async fn release(
    secret: String,
    data: Json<GHRelPayload>,
    mods: &State<Mods>,
    _e: RelGuard,
) -> SimpleResponse {
    if data.0.action == "deleted" {
        return Err(client_err(
            "Deleted releases are ignored by rdb.
To overwrite release information, submit a new release.
To delete your mod from rdb, just contact Dual (Discord ID 303617148411183105).",
        ));
    }

    let submission =
        extract_submission(data.0, secret).ok_or(Custom(Status::BadRequest, "Bad format"))?;

    super::submit::submit(Json(submission), mods).await
}

fn extract_submission(rel: GHRelPayload, secret: String) -> Option<Submission> {
    let (owner, name) = rel.repository.full_name.split_once('/')?;
    let binaries = rel
        .release
        .assets
        .into_iter()
        .map(|a| a.browser_download_url);

    Some(Submission {
        name: name.to_owned(),
        owner: owner.to_owned(),
        secret,
        description: rel.repository.description.unwrap_or_default(),
        homepage: rel.repository.homepage.unwrap_or_default(),
        version: rel.release.tag_name,
        binaries: binaries.collect(),
    })
}
