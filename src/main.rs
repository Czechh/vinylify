pub mod discogs;
pub mod youtube;

use anyhow::Result;
use async_recursion::async_recursion;
use clap::Parser;
use console::{style, Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};
use dotenv::dotenv;
use rspotify::{
    model::{Page, SimplifiedPlaylist, UserId},
    prelude::*,
    ClientCredsSpotify, Credentials,
};

const SPLASH: &str = r#"
VVVVVVVV           VVVVVVVV  iiii                                             lllllll   iiii      ffffffffffffffff
V::::::V           V::::::V i::::i                                            l:::::l  i::::i    f::::::::::::::::f
V::::::V           V::::::V  iiii                                             l:::::l   iiii    f::::::::::::::::::f
V::::::V           V::::::V                                                   l:::::l           f::::::fffffff:::::f
V:::::V           V:::::V iiiiiii nnnn  nnnnnnnn    yyyyyyy           yyyyyyy l::::l iiiiiii   f:::::f       ffffffyyyyyyy           yyyyyyy
V:::::V         V:::::V  i:::::i n:::nn::::::::nn   y:::::y         y:::::y  l::::l i:::::i   f:::::f              y:::::y         y:::::y
V:::::V       V:::::V    i::::i n::::::::::::::nn   y:::::y       y:::::y   l::::l  i::::i  f:::::::ffffff         y:::::y       y:::::y
V:::::V     V:::::V     i::::i nn:::::::::::::::n   y:::::y     y:::::y    l::::l  i::::i  f::::::::::::f          y:::::y     y:::::y
V:::::V   V:::::V      i::::i   n:::::nnnn:::::n    y:::::y   y:::::y     l::::l  i::::i  f::::::::::::f           y:::::y   y:::::y
V:::::V V:::::V       i::::i   n::::n    n::::n     y:::::y y:::::y      l::::l  i::::i  f:::::::ffffff            y:::::y y:::::y
V:::::V:::::V        i::::i   n::::n    n::::n      y:::::y:::::y       l::::l  i::::i   f:::::f                   y:::::y:::::y
V:::::::::V         i::::i   n::::n    n::::n       y:::::::::y        l::::l  i::::i   f:::::f                    y:::::::::y
V:::::::V         i::::::i  n::::n    n::::n        y:::::::y        l::::::li::::::i f:::::::f                    y:::::::y
V:::::V          i::::::i  n::::n    n::::n         y:::::y         l::::::li::::::i f:::::::f                     y:::::y
V:::V           i::::::i  n::::n    n::::n        y:::::y          l::::::li::::::i f:::::::f                    y:::::y
VVV            iiiiiiii  nnnnnn    nnnnnn       y:::::y           lllllllliiiiiiii fffffffff                   y:::::y
"#;

#[derive(Parser)]
#[command(version = "1.0", about = "Vinylify", long_about = None)]
struct Cli {
    #[arg(short, long)]
    username: String,
}

async fn get_user_playlists(
    spotify: &ClientCredsSpotify,
    user_id: &str,
) -> Page<SimplifiedPlaylist> {
    let spotify_user_id = UserId::from_id(user_id).unwrap();
    let limit = 50;
    let offset = 0;
    spotify
        .user_playlists_manual(spotify_user_id, Some(limit), Some(offset))
        .await
        .unwrap()
}

#[async_recursion]
async fn playlist_selection(
    spotify: &ClientCredsSpotify,
    user_id: String,
    theme: ColorfulTheme,
) -> Result<()> {
    let playlist_page = get_user_playlists(spotify, &user_id).await;
    let playlist_names: Vec<String> = playlist_page
        .items
        .iter()
        .map(|item| item.name.clone())
        .collect();

    let selection = Select::with_theme(&theme)
        .with_prompt("Select a playlist")
        .report(false)
        .default(0)
        .items(&playlist_names)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    if selection.is_none() {
        println!("No selection made");
        return Ok(());
    }

    let index = selection.unwrap();
    let playlist_selected_id = playlist_page.items[index].id.clone();
    let playlist_selected_name = playlist_page.items[index].name.clone();
    let playlist = spotify
        .playlist(playlist_selected_id, Some("fields=tracks.items"), None)
        .await;

    let binding = playlist.unwrap();
    let track_list = binding
        .tracks
        .items
        .iter()
        .filter_map(|track_item| {
            if let Some(rspotify::model::PlayableItem::Track(track)) = track_item.track.clone() {
                if let Some(artist) = track.artists.first() {
                    Some((artist.name.clone(), track.name))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<(String, String)>>();

    let user_clone = user_id.clone();
    import_selection(
        spotify,
        user_clone,
        track_list,
        playlist_selected_name,
        theme,
    )
    .await
}

#[async_recursion]
async fn import_selection(
    spotify: &ClientCredsSpotify,
    user_id: String,
    track_list: Vec<(String, String)>,
    playlist_name: String,
    theme: ColorfulTheme,
) -> Result<()> {
    let track_prompt = format!(
        "Would you like to import the following tracks?\n{}",
        track_list
            .iter()
            .fold(String::new(), |acc, (track, artist)| {
                format!("{}{} - {}\n", acc, artist, track)
            })
    );
    let post_selection_options = vec!["Import to Discogs", "YT", "Go back to Playlist list"];
    let post_selection_action = Select::with_theme(&theme)
        .with_prompt(track_prompt)
        .report(false)
        .default(0)
        .items(&post_selection_options)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    if post_selection_action.is_none() {
        println!("No selection made");
        return Ok(());
    }

    let post_selection_index = post_selection_action.unwrap();
    if post_selection_index == 0 {
        println!("Importing tracks");
        let _result = discogs::import_tracks(track_list, &playlist_name).await;
        Ok(())
    } else if post_selection_index == 1 {
        println!("Going to download tracks from YT");
        let _result = youtube::import_tracks(track_list, &playlist_name).await;
        Ok(())
    } else {
        println!("Going back to Playlist list");
        playlist_selection(spotify, user_id, theme).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    println!("{}", style(SPLASH).cyan());
    let theme = ColorfulTheme {
        active_item_style: Style::new().cyan().bold(),
        inactive_item_style: Style::new().white(),
        active_item_prefix: style("> ".to_string()).for_stdout(),
        inactive_item_prefix: style("  ".to_string()).for_stdout(),
        ..ColorfulTheme::default()
    };

    let cli = Cli::parse();
    let creds = Credentials::from_env().unwrap();
    let spotify = ClientCredsSpotify::new(creds);
    spotify.request_token().await.unwrap();

    playlist_selection(&spotify, cli.username.to_string(), theme).await
}
