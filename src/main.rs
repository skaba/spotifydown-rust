mod model;
mod service;


use librespot::core::Error;
use service::download;



use log::LevelFilter;

use crate::model::SpotifyTrack;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_module("librespot", LevelFilter::Debug)
        .init();

    let track = SpotifyTrack {
        id: String::from("49fzPkBb3aOUWYRKaTWVhm")
    };

    download(track).await;


    Ok(())
}
