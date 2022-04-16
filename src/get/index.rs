use rocket::get;

#[get("/")]
pub(crate) async fn index() -> &'static str {
    r#"GET /
Gets this page.

GET /mods/count
Gets the number of mods in the database.

GET /mods/<owner>/<name>
Gets a specific mod. Example response body:
{
    "name": "centipede-shields",
    "owner": "Dual-Iron",
    "updated": 1641861631,
    "downloads": 0,
    "description": "A plugin for Rain World",
    "homepage": "",
    "version": "0.3.0",
    "icon": "https://raw.githubusercontent.com/Dual-Iron/centipede-shields/master/wallpounce_icon.png",
    "binary": "https://github.com/Dual-Iron/centipede-shields/releases/download/0.3.0/CentiShields.dll"
}

GET /mods?<page>&<sort>&<search>
Gets a page of mods. Each page is an array with 20 or fewer elements.
- `page` describes how many pages to skip
- `sort` can be one of `new`, `old`, `most-downloads`, or `least-downloads`
- `search` filters by mods whose names match the query parameter

POST /mods Content-Type=application/json
Submits a mod to the database.
If a mod with the same name+owner already exists, the `secret` key must match as well.
The icon should be a 128x128 PNG file.
Every binary must be either a GitHub release asset, Google Drive file, or a Discord attachment.
Example request body:
{
    "name": "centipede-shields",
    "owner": "Dual-Iron",
    "secret": "not telling you this",
    "description": "A plugin for Rain World",
    "homepage": "",
    "version": "0.3.0",
    "icon": "https://raw.githubusercontent.com/Dual-Iron/centipede-shields/master/wallpounce_icon.png",
    "binary": "https://github.com/Dual-Iron/centipede-shields/releases/download/0.3.0/CentiShields.dll"
}

POST /github?<secret> X-GitHub-Event=ping
Verifies a GitHub webhook for submitting rdb mods automatically.

POST /github?<secret> X-GitHub-Event=release
Submits a mod to the database.
If a mod with the same name+owner already exists, the `secret` key must match as well.
The icon should be a 128x128 PNG file.
This endpoint should be used by GitHub webhooks. Setup: https://user-images.githubusercontent.com/31146412/163689916-df787775-ce33-478e-b7a1-0edac45585dd.mp4

╔═════════════╦═══════════════════════════════════════════════════════╗
║    FIELD    ║                  WHERE IT COMES FROM                  ║
╠═════════════╬═══════════════════════════════════════════════════════╣
║ secret      ║ Given in the URL query parameter.                     ║
╠═════════════╬═══════════════════════════════════════════════════════╣
║ name        ║ The repository's name.                                ║
╠═════════════╬═══════════════════════════════════════════════════════╣
║ owner       ║ The respository's owner.                              ║
╠═════════════╬═══════════════════════════════════════════════════════╣
║ version     ║ The release's tag name excluding any "v" prefix.      ║
╠═════════════╬═══════════════════════════════════════════════════════╣
║ description ║ The repository description at the time of publishing. ║
╠═════════════╬═══════════════════════════════════════════════════════╣
║ icon        ║ The file "icon.png" in the repository's root.         ║
╠═════════════╬═══════════════════════════════════════════════════════╣
║ binary      ║ The release's last asset when sorted alphabetically.  ║
╠═════════════╬═══════════════════════════════════════════════════════╣
║ homepage    ║ The repository's homepage or readme URL.              ║
╚═════════════╩═══════════════════════════════════════════════════════╝
"#
}
