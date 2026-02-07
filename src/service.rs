use crate::model::SpotifyTrack;

use dirs::home_dir;
use std::fs::File;
use std::io::{Read, Seek, Write};

use librespot_audio::{AudioDecrypt, AudioFile};
use librespot_core::{
    Error, SpotifyId, SpotifyUri, cache::Cache, config::SessionConfig, session::Session,
};
use librespot_discovery::Credentials;
use librespot_metadata::{Metadata, Track, audio::AudioFileFormat};

pub async fn download(spotify_track: SpotifyTrack) -> Result<(), Error> {
    let session_config = SessionConfig::default();

    let cache = Cache::new(
        home_dir().unwrap().join(".spotifydown").to_str(),
        None,
        None,
        None,
    )?;
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

    let track_uri = SpotifyUri::from_uri(format!("spotify:track:{}", spotify_track.id).as_str())?;
    let track = Track::get(&session, &track_uri).await?;
    let file = track.files[&AudioFileFormat::OGG_VORBIS_320];
    let track_id: SpotifyId = SpotifyId::from_base62(&spotify_track.id)?;

    let encrypted_file = AudioFile::open(&session, file, 1024 * 1024 * 100).await?;

    let key = match session.audio_key().request(track_id, file).await {
        Ok(key) => Some(key),
        Err(_e) => {
            "Unable to load key, continuing without decryption";
            None
        }
    };

    let mut decrypted_file = AudioDecrypt::new(key, encrypted_file);

    let mut buf: Vec<u8> = Vec::new();
    decrypted_file.seek(std::io::SeekFrom::Start(0xa7));
    let _size: usize = decrypted_file.read_to_end(&mut buf)?;

    let mut file = File::create("hede.ogg")?;
    file.write_all(&mut buf);

    Ok(())
}
