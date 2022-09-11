use crate::discordws::get_spotify_token;
use crate::spotifyws::fetch;
use itertools::Itertools;

mod discordws;
mod model;
mod spotifyws;

fn main() {
    let token = envy::from_env::<model::Config>().unwrap().discord_token;

    println!("Get token by discord...");
    let token = get_spotify_token(&token);

    println!("Spotify Token: {}", token);

    fetch(&token, |v| {
        let v = v.payloads.first().and_then(|v| v.events.first());

        let spotifyws::model::Event::PlayerStateChanged(v) = match v {
            Some(v) => v,
            _ => return,
        };

        if !v.event.state.is_playing {
            println!("Paused");
            return;
        }

        let v = match &v.event.state.item {
            Some(spotifyws::model::PSCItem::Track(x)) => x,
            None => {
                println!("Unknown track");
                return;
            }
        };

        #[allow(unstable_name_collisions)]
        let artists: String = v
            .artists
            .iter()
            .filter_map(|v| match v {
                spotifyws::model::ArtistLikeObject::Artist(v) => Some(v),
            })
            .map(|v| v.name.as_str())
            .intersperse(", ")
            .collect();

        let album = match &v.album {
            spotifyws::model::AlbumLikeObject::Album(a) => a,
            _ => return,
        };

        let albumart: &str = album
            .images
            .iter()
            .max_by_key(|v| v.width * v.height)
            .map(|v| v.url.as_str())
            .unwrap();

        println!("♪ {} {}: {}", v.name, artists, albumart);
    });
}
