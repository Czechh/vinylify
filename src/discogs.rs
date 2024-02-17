use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct GenericResource {
    id: i64,
}

#[derive(Deserialize, Debug)]
struct SearchResult {
    results: Vec<Release>,
}

#[derive(Deserialize, Debug)]
struct Release {
    id: i64,
}

pub async fn create_discogs_folder(folder_name: &str, user: &str, token: &str) -> Result<i64> {
    let client = Client::new();
    let url = format!("https://api.discogs.com/users/{}/collection/folders", user);

    let response = client
        .post(url)
        .header("Authorization", format!("Discogs token={}", token))
        .header("User-Agent", "Vinylify/1.0")
        .json(&serde_json::json!({ "name": folder_name }))
        .send()
        .await?;

    if response.status().is_success() {
        println!("Folder created successfully");
        let folder_response: GenericResource = response.json().await?;

        return Ok(folder_response.id);
    }

    Err(anyhow!("Failed to create folder"))
}

pub async fn search_discogs_track(artist: &str, track_title: &str, token: &str) -> Result<i64> {
    let client = Client::new();
    let url = "https://api.discogs.com/database/search";

    let response = client
        .get(url)
        .header("Authorization", format!("Discogs token={}", token))
        .query(&[
            ("artist", artist.to_string()),
            ("track", track_title.to_string()),
            ("type", "release".to_string()),
        ])
        .header("User-Agent", "Vinylify/1.0")
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        print!("\nSearch result: {}\n", body);
        let search_result: SearchResult = serde_json::from_str(&body)?;

        if let Some(first_release) = search_result.results.first() {
            return Ok(first_release.id);
        }

        return Err(anyhow!("No releases found for the given track"));
    }

    eprintln!("Failed to search. Status: {}", response.status());
    let text = response.text().await?;
    eprintln!("Response text: {}", text);
    Err(anyhow!("Failed to search Discogs"))
}

pub async fn add_release_to_folder(
    folder_id: i64,
    release_id: i64,
    username: &str,
    token: &str,
) -> Result<()> {
    let client = Client::new();
    let url = format!(
        "https://api.discogs.com/users/{}/collection/folders/{}/releases/{}",
        username, folder_id, release_id
    );

    let response = client
        .post(url)
        .header("Authorization", format!("Discogs token={}", token))
        .header("User-Agent", "Vinylify/1.0")
        .send()
        .await?;

    match response.status().as_u16() {
        201 => {
            println!("Release successfully added to the folder.");
        }
        status => {
            eprintln!("Failed to add release to the folder. Status: {}", status);
            let text = response.text().await?;
            eprintln!("Response text: {}", text);
        }
    }

    Ok(())
}

pub async fn import_tracks(track_list: Vec<(String, String)>, playlist_name: &str) -> Result<()> {
    let user = std::env::var("DISCOGS_USERNAME").expect("DISCOGS_USERNAME must be set.");
    let user_token = std::env::var("DISCOGS_TOKEN").expect("DISCOGS_TOKEN must be set.");

    let folder_id = create_discogs_folder(playlist_name, &user, &user_token)
        .await
        .unwrap();

    for (artist, track) in track_list {
        match search_discogs_track(&artist, &track, &user_token).await {
            Ok(release_id) => {
                add_release_to_folder(folder_id, release_id, &user, &user_token)
                    .await
                    .unwrap();
            }
            Err(e) => {
                eprintln!("Failed to add track, skipping: {}", e);
            }
        };
    }

    Ok(())
}
