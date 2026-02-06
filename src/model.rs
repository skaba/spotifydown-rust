use regex::RegexBuilder;
use strum_macros::EnumString;

pub struct SpotifyTrack {
    pub id: String,
}

#[derive(EnumString)]
enum Type {
    #[strum(ascii_case_insensitive)]
    TRACK,
    #[strum(ascii_case_insensitive)]
    ALBUM,
    #[strum(ascii_case_insensitive)]
    PLAYLIST,
    #[strum(ascii_case_insensitive)]
    FILE,
}

struct Url {
    r#type: Type,
    id: String,
}

impl Url {
    fn from_url(url: String) -> Url {
        let regex = RegexBuilder::new(
            r"https?:\/\/[^/]*open\.spotify\.com\/(track|playlist|album)\/([^\s?]+)(\?.*)?",
        )
        .build()
        .unwrap();

        let matches: Vec<_> = regex.find_iter(&url).map(|m| m.as_str()).collect();

        Url {
            r#type: matches[1].to_uppercase().parse::<Type>().unwrap(),
            id: String::from(matches[2]),
        }
    }
}
