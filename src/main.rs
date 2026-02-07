mod model;
mod service;

use service::download;
use model::SpotifyTrack;
use librespot::core::Error;
use log::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_module("librespot", LevelFilter::Info)
        .init();

    let track = SpotifyTrack {
        id: String::from("49fzPkBb3aOUWYRKaTWVhm")
    };

    download(track).await;


    Ok(())
}
