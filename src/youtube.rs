use anyhow::Result;
use reqwest::Error;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Deserialize, Debug)]
pub struct YoutubeSearchResponse {
    pub items: Vec<Item>,
}

#[derive(Deserialize, Debug)]
pub struct Item {
    pub id: VideoId,
    pub snippet: Snippet,
}

#[derive(Deserialize, Debug)]
pub struct VideoId {
    #[serde(rename = "videoId")]
    pub video_id: String,
}

#[derive(Deserialize, Debug)]
pub struct Snippet {
    pub title: String,
}

pub async fn search_youtube(query: &str, api_key: &str) -> Result<Vec<Item>, Error> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://www.googleapis.com/youtube/v3/search")
        .query(&[
            ("part", "snippet"),
            ("q", query),
            ("key", api_key),
            ("type", "video"),
            ("maxResults", "1"),
        ])
        .send()
        .await?
        .json::<YoutubeSearchResponse>()
        .await?;

    Ok(res.items)
}

pub async fn import_tracks(track_list: Vec<(String, String)>, playlist_name: &str) -> Result<()> {
    let api_key = std::env::var("YOUTUBE_API_KEY").expect("YOUTUBE_API_KEY must be set.");
    let dir_path = Path::new(playlist_name);
    fs::create_dir_all(dir_path)?;

    for (artist, track) in track_list {
        let query = format!("{} {}", artist, track);
        match search_youtube(&query, &api_key).await {
            Ok(items) => {
                let first = items.first().unwrap();
                println!("{}: {}", first.id.video_id, first.snippet.title);

                let video_url = format!("https://www.youtube.com/watch?v={}", first.id.video_id);
                let output = format!("{}/%(title)s.%(ext)s", dir_path.to_str().unwrap());
                let status = Command::new("youtube-dl")
                    .args([
                        "-x",
                        "--audio-format",
                        "mp3",
                        "--audio-quality",
                        "0",
                        "-o",
                        &output,
                        &video_url,
                    ])
                    .status()?;

                if !status.success() {
                    eprintln!("youtube-dl command failed for video: {}", video_url);
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }
    Ok(())
}
