# Vinylify

Take a Spotify playlist and turn it into a Discogs collection folder.

## Requirements

- Get Discogs personal token https://www.discogs.com/settings/developers
- Get Spotify client_id and client_secret https://developer.spotify.com/dashboard/applications (callback=`http://localhost:8888/callback`)
- Get YouTube API key https://console.developers.google.com/ (enable YouTube Data API v3)

Set `.env` file with the following:

```env
RSPOTIFY_CLIENT_ID=
RSPOTIFY_CLIENT_SECRET=
RSPOTIFY_REDIRECT_URI=
DISCOGS_TOKEN=
DISCOGS_USERNAME=
YOUTUBE_API_KEY=
```
