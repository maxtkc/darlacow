#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use rocket::Request;

use std::collections::HashMap;

#[get("/")]
fn play() -> Template {
    let mut map: HashMap<u8, u8> = HashMap::with_capacity(0);
    Template::render("play", &map)
}

#[get("/edit")]
fn edit() -> Template {
    let mut map: HashMap<u8, u8> = HashMap::with_capacity(0);
    Template::render("edit", &map)
}

#[get("/docs")]
fn docs() -> Template {
    let mut map: HashMap<u8, u8> = HashMap::with_capacity(0);
    Template::render("docs", &map)
}

#[get("/logs")]
fn logs() -> Template {
    let mut map: HashMap<u8, u8> = HashMap::with_capacity(0);
    Template::render("logs", &map)
}

#[catch(404)]
fn not_found(req: &Request) -> Template {
    let mut map = HashMap::new();
    map.insert("path", req.uri().path());
    Template::render("error/404", &map)
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
        .mount("/", routes![play, edit, docs, logs])
        .attach(Template::fairing())
        .register(catchers![not_found])
}

fn main() {
    rocket().launch();
}
