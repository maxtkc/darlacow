#![feature(proc_macro_hygiene, decl_macro)]

// the server engine itself
#[macro_use] extern crate rocket;
// for tera templates and other rocket extras
extern crate rocket_contrib;

// serialize/deserialize json (strings <-> json)
#[macro_use] extern crate serde_json;

// crate for gpio pins on raspberry pi
extern crate rppal;
// serial
extern crate serialport;

// sqlite database client
extern crate rusqlite;

// music player dameon client for rust
extern crate mpd;

// Static files and jinja template engine
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

// Request struct
use rocket::Request;

// For passing to templates
use std::collections::HashMap;
// General
use std::path::Path;
// Threading
use std::{thread, time};

// Rust sqlite crate
use rusqlite::types::ToSql;
use rusqlite::{Connection, NO_PARAMS};

// Serialize and deserialize json
use serde_json::Value;

// High level rapsberry pi peripheral access
use rppal::i2c::I2c;

// For serial to arduino
use serialport::prelude::*;

// Crate to communicate with Music Player Dameon
use mpd::{Client,Song};

/// Names of each of the relays
/// Set these names to the names of the physical devices
/// **Set them in the order that they are wired to the board**
const RELAYS: &'static [&'static str] = &["Cow Up", "Main Motion", "Strobe",  "Monkeys",  "Marque Lights",  "Masks",  "Center Lights",  "Disco", "Vomit Light"];
const RELAYS_DEFAULT: &'static [&'static str] = &["Relay 1", "Relay 2", "Relay 3", "Relay 4", "Relay 5", "Relay 6", "Relay 7", "Relay 8", "Relay 9", "Relay 10", "Relay 11", "Relay 12", "Relay 13", "Relay 14", "Relay 15", "Relay 16"];
const MAIN_MOT_I: &'static &usize = & &1; // The index of main_motion in the RELAYS array
const MAIN_MOT_SLEEP: &'static u64 = &200; // Main motion requires the on off thing (in milliseconds)

// MCP23017 I2C default slave address.
const ADDR_MPC23017: u16 = 0x20;

// MCP23017 Registers
const MCP23017_IODIRA: u8 = 0x00; // data direction
const MCP23017_GPIOA: u8 = 0x12; // ports

/// Names of each of the songs TODO: dynamically grab from songs dir
const SONGS: &'static [&'static str] = &["", "BYARD.mp3", "CRACK.mp3", "ICE-C.mp3", "LONE.mp3", "LOONY.mp3", "MLING.mp3", "MTC.mp3", "NEWMC.mp3", "MARV.mp3", "SPACE.mp3"];


/// Home route --------------------------------------------------------

#[get("/")]
fn home() -> Template {
    let mut map = HashMap::new();

    // Gets sequences to list as options to play
    let json: Value = serde_json::from_str(&get_seqs()).unwrap();
    map.insert("seqs", &json);
    Template::render("home", &map)
}


/// Stop route --------------------------------------------------------

#[get("/stop")]
fn stop() -> () {
    // Kill MPD
    let mut mpdconn = Client::connect("127.0.0.1:6600").unwrap();
    mpdconn.stop();

    // create i2c instance
    let mut i2c = I2c::new().unwrap();

    // Set the I2C slave address to the device we're communicating with.
    i2c.set_slave_address(ADDR_MPC23017).unwrap();

    // set relays to outputs
    i2c.block_write(MCP23017_IODIRA, &[0, 0]).unwrap();

    // turn off all relays
    i2c.block_write(MCP23017_GPIOA, &[0xFF, 0xFF]).unwrap();

    let port_name = "/dev/ttyACM0";
    let baud_rate = 9600;
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = time::Duration::from_millis(10);
    settings.baud_rate = baud_rate;
    let mut port = serialport::open_with_settings(&port_name, &settings).unwrap();
    port.write("S".as_bytes());
}


/// Play route --------------------------------------------------------

#[get("/play/<name>")]
fn play(name: String) -> String {
    // Create a connection to the sql database
    let sqlconn = Connection::open(Path::new("db/sequences.db")).unwrap();

    // Get the json of the sequence with name 'name' from the database
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
        println!("setting volume");
        mpdconn.volume(100).unwrap_or_default(); // doesn't work for some reason
        println!("set volume");

        song_i = 0;

        // create i2c instance
        let mut i2c = I2c::new().unwrap();
        println!("created i2c instance");

        // Set the I2C slave address to the device we're communicating with.
        i2c.set_slave_address(ADDR_MPC23017).unwrap();

        // set relays to outputs
        i2c.block_write(MCP23017_IODIRA, &[0, 0]).unwrap();
        println!("set relays to outputs");

        // open serial port
        //let port_name = &serialport::available_ports().unwrap()[0].port_name;
        let port_name = "/dev/ttyACM0";
        let baud_rate = 9600;
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = time::Duration::from_millis(10);
        settings.baud_rate = baud_rate;
        let mut port = serialport::open_with_settings(&port_name, &settings).unwrap();
        println!("opened serial port");
        port.write("S".as_bytes());

        println!("\n\nplaying data:\n{}\n", json);
        let mut i = 0;
        while json[i] != Value::Null {
            println!("row: {}", json[i]);
            // song
            if json[i]["song"].as_str().unwrap_or_default() != "" {
                println!("Playing song {}", song_i);
                if song_i > 0 {
                    println!("next");
                    mpdconn.next();
                } else {
                    println!("play");
                    mpdconn.play();
                }
                song_i += 1;
            }
            
            // relays
            let mut relay_val: u16 = 0;
            for (j, relay) in RELAYS.iter().enumerate() {
                if json[i][relay].as_bool().unwrap_or_default() {
                    println!("turning on {}", relay);
                    relay_val += 1 << j;
                }
            }
            println!("Relays set to {}", relay_val);
            i2c.block_write(
                MCP23017_GPIOA,
                &[!(relay_val & 0xFF) as u8, !(relay_val >> 8) as u8],
                ).unwrap();
            
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
                relay_val &= !(1 << **MAIN_MOT_I);
                i2c.block_write(
                    MCP23017_GPIOA,
                    &[!(relay_val & 0xFF) as u8, !(relay_val >> 8) as u8],
                    ).unwrap();
                thread::sleep(time::Duration::from_millis(
                        json[i]["time"].as_u64().unwrap()
                        - *MAIN_MOT_SLEEP));
            } else {
                thread::sleep(time::Duration::from_millis(
                        json[i]["time"].as_u64().unwrap()));
            }
            i += 1;
        }
        println!("stop");
        mpdconn.stop();
        i2c.block_write(MCP23017_GPIOA, &[0xFF, 0xFF]).unwrap();
        port.write("S".as_bytes());
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


/// Test route --------------------------------------------------------

#[get("/test")]
fn test() -> Template {
    let mut map = HashMap::new();
    let mut relays: [&str; 16] = ["hello world";16];
    for i in 0..RELAYS.len() {
        relays[i] = &RELAYS[i];
    }
    for i in RELAYS.len()..relays.len() {
        relays[i] = &RELAYS_DEFAULT[i];
    }
    map.insert("relays", &relays);
    Template::render("test", &map)
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

#[get("/set_relays/<relay_val>")]
fn set_relays(relay_val: u16) -> String {
    // create i2c instance
    let mut i2c = I2c::new().unwrap();
    println!("created i2c instance");

    // Set the I2C slave address to the device we're communicating with.
    i2c.set_slave_address(ADDR_MPC23017).unwrap();

    // set relays to outputs
    i2c.block_write(MCP23017_IODIRA, &[0, 0]).unwrap();
    println!("set relays to outputs");

    println!("Relays set to {}", relay_val);
    i2c.block_write(
        MCP23017_GPIOA,
        &[!(relay_val & 0xFF) as u8, !(relay_val >> 8) as u8],
        ).unwrap();
    // i2c.block_write(MCP23017_GPIOA, &[0xFF, 0xFF]).unwrap();

    format!("set relays to {0} ({0:b})", relay_val)
}

#[get("/secondary_motion/<serial_val>")]
fn secondary_motion(serial_val: String) -> String {
        // open serial port
        //let port_name = &serialport::available_ports().unwrap()[0].port_name;
        let port_name = "/dev/ttyACM0";
        let baud_rate = 9600;
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = time::Duration::from_millis(10);
        settings.baud_rate = baud_rate;
        let mut port = serialport::open_with_settings(&port_name, &settings).unwrap();
        port.write(serial_val.as_bytes());
    format!("Sent {} over serial", serial_val)
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
               test,
               docs,
               logs,
               get_seqs,
               get_seq,
               new_seq,
               set_seq,
               set_relays,
               secondary_motion,
               play,
               stop])
        .attach(Template::fairing())
        .register(catchers![not_found])
}

fn main() {
    rocket().launch();
}
