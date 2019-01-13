#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
extern crate rocket_contrib;

#[macro_use] extern crate serde_json;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use rocket::Request;

use std::collections::HashMap;
use std::path::Path;
use std::{thread, time};

use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};

use serde_json::Value;
use serde_json::Number;

/// Names of each of the relays
const RELAYS: &'static [&'static str] = &["Relay 1", "Relay 2", "Relay 3"];
//const RELAYS: &'static [(u8, &'static str)] = &[
//    (15, "Relay 1"),
//    (11, "Relay 2"),
//    (16, "Relay 3")];

/// Names of each of the songs
const SONGS: &'static [&'static str] = &["", "Song 1", "Song 2", "Song 3"];


/// Home route --------------------------------------------------------

#[get("/")]
fn home() -> Template {
    let mut map = HashMap::new();
    let json: Value = serde_json::from_str(&get_seqs()).unwrap();
    map.insert("seqs", &json);
    Template::render("home", &map)
}


/// Play route --------------------------------------------------------

#[get("/play/<name>")]
fn play(name: String) -> String {
    let conn = Connection::open(Path::new("db/sequences.db")).unwrap();
    let seq: String = conn.query_row("SELECT data FROM seq WHERE name = ?1",
                   &[&name], |row| { row.get(0) }).unwrap();
    // Creates json variable that can be used dynamically ie. json[0]["lksdjf"]
    let json: Value = serde_json::from_str(&seq).unwrap();
    println!("{}", serde_json::to_string(&json).unwrap());

    let mut i = 0;
    let mut sum = 0;
    while json[i] != Value::Null {
        println!("{}", json[i]["time"]);
        sum += json[i]["time"].as_u64().unwrap();
        i += 1;
    }
    thread::spawn(move || {
        println!("\n\nplaying data:\n{}\n", json);
        let mut i = 0;
        while json[i] != Value::Null {
            println!("row: {}", json[i]);
            // song
            
            // relays
            for (j, relay) in RELAYS.iter().enumerate() {
                if json[i][relay].as_bool().unwrap_or_default() == true {

                    println!("turning on {}", relay);
                }
            }
            // secondary motion
            // wait
            thread::sleep(time::Duration::from_millis(json[i]["time"].as_u64().unwrap()));
            i += 1;
        }
        println!("done with sequence\n------------------------\n");
    });
    return sum.to_string();
}


/// Edit route --------------------------------------------------------

#[get("/edit")]
fn edit() -> Template {
    let mut map = HashMap::new();
    map.insert("relays", &RELAYS);
    map.insert("songs", &SONGS);
    Template::render("edit", &map)
}


/// Docs route --------------------------------------------------------

#[get("/docs")]
fn docs() -> Template {
    let mut map: HashMap<u8, u8> = HashMap::with_capacity(0);
    Template::render("docs", &map)
}


/// Logs route --------------------------------------------------------

#[get("/logs")]
fn logs() -> Template {
    let mut map: HashMap<u8, u8> = HashMap::with_capacity(0);
    Template::render("logs", &map)
}

/// Helper routes --------------------------------------------------------

#[get("/get_seqs")]
fn get_seqs() -> String {
    let conn = Connection::open(Path::new("db/sequences.db"))
        .unwrap();
    let mut stmt = conn.prepare("SELECT name FROM seq")
        .unwrap();
    let name_iter = stmt
        .query_map(NO_PARAMS,
                   |row| { StringStruct { data: row.get(0) } } )
        .unwrap();
    let mut names: Vec<String> = Vec::new();
    for name in name_iter {
        names.push(name.unwrap().data);
    }
    serde_json::to_string(&json!(names))
        .unwrap()
}

/// Workaround for get_seqs function to get names of seqs
#[derive(Debug)]
struct StringStruct {
    data: String
}

#[get("/get_seq/<name>")]
fn get_seq(name: String) -> String {
    let conn = Connection::open(Path::new("db/sequences.db")).unwrap();
    match conn.query_row("SELECT data FROM seq WHERE name = ?1",
                   &[&name], |row| { row.get(0) }) {
        Ok(json) => json,
        Err(_e) => String::from("[{\"time\":1000, \"sec_mot\":\"\"}]")
    }
}

#[get("/new_seq/<name>")]
fn new_seq(name: String) -> String {
    let data = String::from("[{\"time\":1000, \"sec_mot\":\"\"}]");
    let conn = Connection::open(Path::new("db/sequences.db")).unwrap();
    conn.execute("INSERT INTO seq (name, data) VALUES (?1, ?2)",
                   &[&name, &data]).unwrap();
    String::from("created new sequence")
}

#[post("/set_seq/<name>", data = "<data>")]
fn set_seq(data: String, name: String) -> &'static str {
    let conn = Connection::open(Path::new("db/sequences.db")).unwrap();
    conn.execute( // Delete any previous sequence
        "DELETE FROM seq WHERE
                  name = ?1",
                  &[&name as &ToSql]
                  ).unwrap();
    conn.execute( // Make a new sequence
        "INSERT INTO seq (name, data)
                  VALUES (?1, ?2)",
                  &[&name as &ToSql, &data as &ToSql]
                  ).unwrap();
    "setting"
}

#[catch(404)]
fn not_found(req: &Request) -> Template {
    let mut map = HashMap::new();
    map.insert("path", req.uri().path());
    Template::render("error/404", &map)
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", StaticFiles::from("static"))
        .mount("/", routes![
               home,
               edit,
               docs,
               logs,
               get_seqs,
               get_seq,
               new_seq,
               set_seq,
               play])
        .attach(Template::fairing())
        .register(catchers![not_found])
}

fn main() {
    rocket().launch();
}
