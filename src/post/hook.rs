use crate::prelude::*;
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};

#[rocket::post("/", format = "application/json")]
pub(crate) async fn ping(_e: PingGuard) -> &'static str {
    "Successfully connected to rdb! Make sure that:
- You set content type to \"application/json\",
- You enabled Release events, and
- SSL verification is enabled.

With all that in mind, the next release you create/edit will be published.
The last asset in that release will be the binary that's uploaded to rdb.
"
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
To delete your mod from rdb, contact Dual (Discord ID 303617148411183105).",
        ));
    }

    let submission = extract_submission(data.0, secret).ok_or(Custom(
        Status::BadRequest,
        "Bad format. Did you have a release asset?",
    ))?;

    super::submit::submit(Json(submission), mods).await
}

fn extract_submission(mut rel: GHRelPayload, secret: String) -> Option<Submission> {
    let (owner, name) = rel.repository.full_name.split_once('/')?;
    let binary = rel.release.assets.pop()?.browser_download_url;

    let mut homepage = rel.repository.homepage.unwrap_or_default();
    if homepage.is_empty() {
        homepage = format!("https://github.com/{}#readme", rel.repository.full_name);
    }

    Some(Submission {
        name: name.to_owned(),
        owner: owner.to_owned(),
        secret,
        description: rel.repository.description.unwrap_or_default(),
        homepage,
        icon: format!(
            "https://raw.githubusercontent.com/{}/{}/icon.png",
            rel.repository.full_name, rel.release.tag_name
        ),
        version: rel.release.tag_name,
        binary,
    })
}
