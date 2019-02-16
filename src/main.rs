#![feature(proc_macro_hygiene, decl_macro)]

// the server engine itself
#[macro_use] extern crate rocket;
// for tera templates and other rocket extras
extern crate rocket_contrib;

// serialize/deserialize json (strings <-> json)
#[macro_use] extern crate serde_json;

// crate for gpio pins on raspberry pi
extern crate rust_gpiozero;
// serial
extern crate serialport;

// sqlite database client
extern crate rusqlite;

// music player dameon client for rust
extern crate mpd;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

use rocket::Request;

use std::collections::HashMap;
use std::path::Path;
use std::{thread, time};
use std::process;

use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};

use serde_json::Value;

use rust_gpiozero::LED;
use rust_gpiozero::OutputDeviceTrait;

use serialport::prelude::*;

use mpd::{Client,Song};

/// Names of each of the relays
const RELAYS: &'static [&'static str] = &["Relay 1", "Relay 2", "Relay 3",  "Relay 4",  "Relay 5",  "Relay 6",  "Relay 7",  "Main Motion"];
const RELAY_PINS: &'static [u64] = &[14, 15, 23, 18, 16, 20, 12, 21];
const MAIN_MOT_I: &'static &usize = & &7;
const MAIN_MOT_SLEEP: &'static u64 = &200;

/// Names of each of the songs TODO: dynamically grab from songs dir
const SONGS: &'static [&'static str] = &["", "West Side Story [10] Intermission.m4a", "West Side Story [11] I feel Pretty.m4a", "West Side Story [12] One hand, One heart.m4a", "West Side Story [13] Quintet.m4a"];


/// Home route --------------------------------------------------------

#[get("/")]
fn home() -> Template {
    let mut map = HashMap::new();
    let json: Value = serde_json::from_str(&get_seqs()).unwrap();
    map.insert("seqs", &json);
    Template::render("home", &map)
}


/// Stop route --------------------------------------------------------

#[get("/stop")]
fn stop() -> () {
    let mut mpdconn = Client::connect("127.0.0.1:6600").unwrap();
    mpdconn.stop();
    process::exit(1);
}


/// Play route --------------------------------------------------------

#[get("/play/<name>")]
fn play(name: String) -> String {
    let sqlconn = Connection::open(Path::new("db/sequences.db")).unwrap();
    let seq: String = sqlconn.query_row("SELECT data FROM seq WHERE name = ?1",
                   &[&name], |row| { row.get(0) }).unwrap();
    // Creates json variable that can be used dynamically ie. json[0]["lksdjf"]
    let json: Value = serde_json::from_str(&seq).unwrap();
    println!("{}", serde_json::to_string(&json).unwrap());

    // get total time to return
    let mut i = 0;
    let mut sum = 0;
    while json[i] != Value::Null {
        println!("{}", json[i]["time"]);
        sum += json[i]["time"].as_u64().unwrap();
        i += 1;
    }

    // start thread to play
    thread::spawn(move || {
        // load music
        let mut mpdconn = Client::connect("127.0.0.1:6600").unwrap();
        mpdconn.clear();
        let mut song_i = 0;
        while json[song_i] != Value::Null {
            println!("{}", json[song_i]["song"]);
            let mut file = json[song_i]["song"].to_string();
            file.remove(0);
            file.remove(file.capacity() - 2);
            mpdconn.push(Song {
                file: format!("darlacow/songs/{}", file),
                         ..Default::default()
            });
            song_i += 1;
        }
        mpdconn.volume(100).unwrap_or_default();
        song_i = 0;

        // create list of relays
        let mut relay_devs: Vec<LED> = Vec::new();
        for pin in RELAY_PINS {
            relay_devs.push(LED::new(*pin));
        }

        // open serial port
        //let port_name = &serialport::available_ports().unwrap()[0].port_name;
        let port_name = "/dev/ttyACM0";
        let baud_rate = 9600;
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = time::Duration::from_millis(10);
        settings.baud_rate = baud_rate;
        let mut port = serialport::open_with_settings(&port_name, &settings).unwrap();
        port.write("S".as_bytes());

        println!("\n\nplaying data:\n{}\n", json);
        let mut i = 0;
        while json[i] != Value::Null {
            println!("row: {}", json[i]);
            // song
            if json[i]["song"].as_str().unwrap_or_default() != "" {
                println!("Playing song {}", song_i);
                if song_i > 0 {
                    mpdconn.next();
                } else {
                    mpdconn.play();
                }
                song_i += 1;
            }
            
            // relays
            for (j, relay) in RELAYS.iter().enumerate() {
                if json[i][relay].as_bool().unwrap_or_default() == true {
                    println!("turning on {}", relay);
                    relay_devs[j].off();
                } else {
                    println!("turning off {}", relay);
                    relay_devs[j].on();
                }
            }
            // secondary motion
            let sec_mot = json[i]["sec_mot"].as_str().unwrap_or_default();
            println!("secondary motion {}", sec_mot);
            if sec_mot == "out" {
                port.write("F".as_bytes());
            } else if sec_mot == "in" {
                port.write("R".as_bytes());
            }

            // wait
            // edge case: main motion
            if json[i][MAIN_MOT_I].as_bool().unwrap_or_default() {
                thread::sleep(time::Duration::from_millis(
                        *MAIN_MOT_SLEEP));
                relay_devs[**MAIN_MOT_I].off();
                thread::sleep(time::Duration::from_millis(
                        json[i]["time"].as_u64().unwrap()
                        - *MAIN_MOT_SLEEP));
            }
            thread::sleep(time::Duration::from_millis(
                    json[i]["time"].as_u64().unwrap()));
            i += 1;
        }
        mpdconn.stop();
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
    let map: HashMap<u8, u8> = HashMap::with_capacity(0);
    Template::render("docs", &map)
}


/// Logs route --------------------------------------------------------

#[get("/logs")]
fn logs() -> Template {
    let map: HashMap<u8, u8> = HashMap::with_capacity(0);
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
               play,
               stop])
        .attach(Template::fairing())
        .register(catchers![not_found])
}

fn main() {
    rocket().launch();
}
