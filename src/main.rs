mod get;
mod guards;
mod indexes;
mod post;
mod prelude;
mod serde;

use mongodb::{options::ClientOptions, Client};
use rocket::*;

#[launch]
async fn rocket() -> _ {
    println!("Connecting MongoDB client");

    let conn = std::env::var("DB_URL").expect("no `DB_URL` env var");
    let options = ClientOptions::parse(conn)
        .await
        .expect("invalid connection url");

    let client = Client::with_options(options).expect("failed to connect MongoDB client");
    let mods: prelude::Mods = client.database("test2").collection("mods");

    indexes::add_indexes(&mods).await;

    println!("Launching Rocket");

    build()
        .manage(mods)
        .mount("/", routes![get::index::index])
        .mount("/github", routes![post::hook::ping, post::hook::release])
        .mount(
            "/mods",
            routes![
                get::one_mod::one_mod,
                get::many_mods::many_mods,
                post::submit::submit
            ],
        )
}
