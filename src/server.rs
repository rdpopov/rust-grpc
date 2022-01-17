use rusqlite::{params, Connection, Result};
use std::sync::Mutex;
// use tonic::codegen::http::request;
use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::AddResult;
use hello_world::{SongMeta, SongName};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Debug)]
pub struct MyGreeter {
    db: Mutex<Connection>,
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    /// Add metadata for a song of filename fname
    /// returns 1 if song does not exit and was successfull
    ///         0 otherwise
    ///
    /// * `request`:
    async fn add_meta(&self, request: Request<SongMeta>) -> Result<Response<AddResult>, Status> {
        let req = request.into_inner();
        let res = self.db.lock().unwrap().execute(
            "INSERT INTO songs (fname, name, artist, album, artwork, lyrics) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![req.fname ,req.name, req.artist, req.album, req.artwork, req.lyrics],
        );

        let reply = AddResult {
            result: match res {
                Err(_) => "0".to_string(),
                Ok(_) => "1".to_string(),
            },
        };

        Ok(Response::new(reply))
    }

    /// Update metadata for a song of filename fname
    /// returns 0 if song does not exit or failed somehow
    ///         1 otherwise
    ///
    /// * `request`: 
    async fn update_meta(&self, request: Request<SongMeta>) -> Result<Response<AddResult>, Status> {
        let req = request.into_inner();
        let res = self.db.lock().unwrap().execute(
            "UPDATE songs SET name = ?1, artist = ?2, album = ?3, artwork = ?4, lyrics = ?4 WHERE fname = ?6",
            params![req.name, req.artist, req.album, req.artwork, req.lyrics, req.fname],
        );

        let reply = AddResult {
            result: match res {
                Err(_) => "0".to_string(),
                Ok(_) => "1".to_string(),
            },
        };

        Ok(Response::new(reply))
    }

    /// Query metadta by using filename
    ///
    /// * `request`:
    async fn query_meta(&self, request: Request<SongName>) -> Result<Response<SongMeta>, Status> {
        println!("Got a request: {:?}", request);
        let query = format!(
            // TODO: maybe make it so that this uses name as well, can be beneficial
            "SELECT fname, name, artist, album, artwork, lyrics FROM songs WHERE fname = \"{}\"",
            request.into_inner().song_name
        );
        println!("{}", query);

        let db = self.db.lock().unwrap();
        let mut stmt = db.prepare(&query).unwrap();
        let song_iter = stmt.query_map([], |row| {
            Ok(SongMeta {
                fname: row.get(0)?,
                name: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                artwork: row.get(4)?,
                lyrics: row.get(5)?,
            })
        });
        let res = song_iter.unwrap().next();
        match res {
            None => {
                return Ok(Response::new(SongMeta {
                    fname: "None".to_string(),
                    name: "None".to_string(),
                    artist: "None".to_string(),
                    album: "None".to_string(),
                    artwork: "None".to_string(),
                    lyrics: "None".to_string(),
                }))
            }
            _ => return Ok(Response::new(res.unwrap().unwrap())),
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = MyGreeter {
        db: Mutex::new(Connection::open("song_meta_db.db").unwrap()), // hope it does not break ...
    };

    greeter.db.lock().unwrap().execute(
        "CREATE TABLE IF NOT EXISTS songs (
            fname   TEXT PRIMARY KEY,
            name    TEXT NOT NULL,
            artist  TEXT NOT NULL,
            album   TEXT NOT NULL,
            artwork TEXT NOT NULL,
            lyrics  TEXT NOT NULL)",
        [],
    )?;

    println!("Running ... ");

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;
    Ok(())
}
