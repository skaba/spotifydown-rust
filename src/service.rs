use std::fs::File;
use std::io::{Read, Seek, Write};

use crate::model::SpotifyTrack;

use librespot::{
    audio::{AudioDecrypt, AudioFile},
    core::{Error, SpotifyId, SpotifyUri, cache::Cache, config::SessionConfig, session::Session},
    discovery::Credentials,
    metadata::{Metadata, Track, audio::AudioFileFormat},
};

const CACHE: &str = ".cache";
const SPOTIFY_OGG_HEADER_END: u64 = 0xa7;

pub async fn download(spotify_track: SpotifyTrack) -> Result<(), Error> {
    let session_config = SessionConfig::default();
    //let player_config = PlayerConfig::default();
    //let audio_format = AudioFormat::default();
    //let connect_config = ConnectConfig::default();
    //let mixer_config = MixerConfig::default();
    //let request_options = LoadRequestOptions::default();

    let cache = Cache::new(Some(CACHE), None, None, None)?;
    let credentials = cache
        .credentials()
        .ok_or(Error::unavailable("credentials not cached"))
        .or_else(|_| {
            librespot_oauth::OAuthClientBuilder::new(
                &session_config.client_id,
                "http://127.0.0.1:8898/login",
                vec![
                    "app-remote-control",
                    "playlist-modify",
                    "playlist-modify-private",
                    "playlist-modify-public",
                    "playlist-read",
                    "playlist-read-collaborative",
                    "playlist-read-private",
                    "streaming",
                    "ugc-image-upload",
                    "user-follow-modify",
                    "user-follow-read",
                    "user-library-modify",
                    "user-library-read",
                    "user-modify",
                    "user-modify-playback-state",
                    "user-modify-private",
                    "user-personalized",
                    "user-read-birthdate",
                    "user-read-currently-playing",
                    "user-read-email",
                    "user-read-play-history",
                    "user-read-playback-position",
                    "user-read-playback-state",
                    "user-read-private",
                    "user-read-recently-played",
                    "user-top-read",
                ],
            )
            .open_in_browser()
            .build()?
            .get_access_token()
            .map(|t| Credentials::with_access_token(t.access_token))
        })?;

    let session = Session::new(session_config, Some(cache));
    session.connect(credentials, true).await?;

    //format!("spotify:track:{}", track.id);

    let track_uri = SpotifyUri::from_uri(format!("spotify:track:{}", spotify_track.id).as_str())?;
    let track = Track::get(&session, &track_uri).await?;
    let file = track.files[&AudioFileFormat::OGG_VORBIS_320];
    let track_id: SpotifyId = SpotifyId::from_base62(&spotify_track.id)?;

    let encrypted_file = AudioFile::open(&session, file, 1024 * 1024).await?;

    //let is_cached = encrypted_file.is_cached();

    //let stream_loader_controller = encrypted_file.get_stream_loader_controller().ok()?;

    // Not all audio files are encrypted. If we can't get a key, try loading the track
    // without decryption. If the file was encrypted after all, the decoder will fail
    // parsing and bail out, so we should be safe from outputting ear-piercing noise.
    let key = session.audio_key().request(track_id, file).await?;

    let mut decrypted_file = AudioDecrypt::new(Some(key), encrypted_file);

    let mut buf: Vec<u8> = Vec::new();
    decrypted_file.seek(std::io::SeekFrom::Start(SPOTIFY_OGG_HEADER_END));
    let _size: usize = decrypted_file.read_to_end(&mut buf)?;

    let mut file = File::create("hede.ogg")?;
    file.write_all(&mut buf);

    Ok(())
}
