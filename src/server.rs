use rusqlite::{params, Connection, Result};
use std::sync::Mutex;
use tonic::codegen::http::request;
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
    // db: Mutex<Box<Connection>>,
}

struct SongEntry {
    id: i32,
    name: String,
    data: Option<Vec<u8>>,
}


#[tonic::async_trait]
impl Greeter for MyGreeter {
    // This Method implements adding data to the db
    async fn add_meta(&self, request: Request<SongMeta>) -> Result<Response<AddResult>, Status> {
        let req = request.into_inner();
        // NOTE: this is iffy does the mutex drop the guard when the request is done?
        // or do i have to free it>
        let res = self.db.lock().unwrap().execute(
            "INSERT INTO songs (fname, name, artist, album, artwork, lyrics) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![req.fname ,req.name, req.artist, req.album, req.artwork, req.lyrics],
        );
        let reply = hello_world::AddResult {
            result: format!("Done {}", res.unwrap()),
        };
        Ok(Response::new(reply))
    }
    async fn query_meta(&self, request: Request<SongName>) -> Result<Response<SongMeta>, Status> {
        println!("Got a request: {:?}", request);
        // TODO: do something with request.name
        let query = "SELECT name, artist, album, artwork, lyrics FROM songs WHERE songs.fname = {}".format(request.into_inner().song_name);

        let db = self.db.lock().unwrap();
        let stmt = db.prepare(&query);


        let reply = hello_world::SongMeta {
            name: String::from("name of song"),
            artist: String::from("artist "),
            album: String::from("album name "),
            artwork: String::from("artwork location "),
            lyrics: String::from("Lyrics"),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    // TODO: add a data structure for song meta and create table
    let greeter = MyGreeter {
        db: Mutex::new(Connection::open("song_meta_db.db").unwrap()), // hope it does not break ...
    };

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
