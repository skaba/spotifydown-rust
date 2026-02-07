use std::fs::File;
use std::io::{Read, Seek, Write};

use crate::model::SpotifyTrack;

use librespot::discovery::Credentials;
use librespot::{
    audio::{AudioDecrypt, AudioFile},
    core::{Error, SpotifyId, SpotifyUri, cache::Cache, config::SessionConfig, session::Session},
    metadata::{Metadata, Track, audio::AudioFileFormat, audio::AudioItem},
};

const CACHE: &str = "~/.cache";
const SPOTIFY_OGG_HEADER_END: u64 = 0xa7;

pub async fn download(spotify_track: SpotifyTrack) {
    let session_config = SessionConfig::default();
    //let player_config = PlayerConfig::default();
    //let audio_format = AudioFormat::default();
    //let connect_config = ConnectConfig::default();
    //let mixer_config = MixerConfig::default();
    //let request_options = LoadRequestOptions::default();

    let cache = Cache::new(Some(CACHE), None, None, None).unwrap();
    let credentials = cache
        .credentials()
        .ok_or(Error::unavailable("credentials not cached"))
        .or_else(|_| {
            librespot_oauth::OAuthClientBuilder::new(
                &session_config.client_id,
                "http://127.0.0.1:8898/login",
                vec!["streaming"],
            )
            .open_in_browser()
            .build()?
            .get_access_token()
            .map(|t| Credentials::with_access_token(t.access_token))
        })
        .unwrap();

    let session = Session::new(session_config, Some(cache));
    session.connect(credentials, true).await.unwrap();

    //format!("spotify:track:{}", track.id);

    let track_uri =
        SpotifyUri::from_uri(format!("spotify:track:{}", spotify_track.id).as_str()).unwrap();
    let track = Track::get(&session, &track_uri).await.unwrap();
    let file = track.files[&AudioFileFormat::OGG_VORBIS_320];
    let track_id = SpotifyId::from_base62(&spotify_track.id).unwrap();

    let encrypted_file = AudioFile::open(&session, file, 1024 * 1024).await.unwrap();

    //let is_cached = encrypted_file.is_cached();

    //let stream_loader_controller = encrypted_file.get_stream_loader_controller().ok()?;

    // Not all audio files are encrypted. If we can't get a key, try loading the track
    // without decryption. If the file was encrypted after all, the decoder will fail
    // parsing and bail out, so we should be safe from outputting ear-piercing noise.
    let key = session.audio_key().request(track_id, file).await.unwrap();

    let mut decrypted_file = AudioDecrypt::new(Some(key), encrypted_file);

    let mut buf: Vec<u8> = Vec::new();
    decrypted_file.seek(std::io::SeekFrom::Start(SPOTIFY_OGG_HEADER_END));
    let _size = decrypted_file.read_to_end(&mut buf).unwrap();

    let mut file = File::create("hede.ogg").unwrap();
    file.write_all(&mut buf);
}
