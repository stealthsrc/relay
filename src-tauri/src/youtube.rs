use std::collections::HashMap;

use anyhow::{Result, bail};
use reqwest::Client;
use serde::Deserialize;

const SEARCH_ENDPOINT: &str = "https://www.googleapis.com/youtube/v3/search";
const VIDEOS_ENDPOINT: &str = "https://www.googleapis.com/youtube/v3/videos";
const MAX_TRACK_DURATION_SECONDS: u64 = 180;
const SEARCH_CANDIDATE_LIMIT: usize = 25;
const RESULT_LIMIT: usize = 10;
const MAX_QUERY_CHARS: usize = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YouTubeTrack {
    pub video_id: String,
    pub title: String,
    pub channel_title: String,
    pub thumbnail: String,
    pub duration_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    #[serde(default)]
    items: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    id: SearchId,
    snippet: VideoSnippet,
}

#[derive(Debug, Deserialize)]
struct SearchId {
    #[serde(rename = "videoId")]
    video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VideoResponse {
    #[serde(default)]
    items: Vec<VideoItem>,
}

#[derive(Debug, Deserialize)]
struct VideoItem {
    id: String,
    #[serde(rename = "contentDetails")]
    content_details: VideoContentDetails,
}

#[derive(Debug, Deserialize)]
struct VideoContentDetails {
    duration: String,
}

#[derive(Debug, Deserialize)]
struct VideoSnippet {
    title: String,
    #[serde(rename = "channelTitle")]
    channel_title: String,
    thumbnails: ThumbnailSet,
}

#[derive(Debug, Deserialize)]
struct ThumbnailSet {
    default: Option<Thumbnail>,
    medium: Option<Thumbnail>,
    high: Option<Thumbnail>,
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: String,
}

pub async fn search(query: &str, api_key: &str) -> Result<Vec<YouTubeTrack>> {
    let query = normalize_query(query)?;
    if api_key.trim().is_empty() {
        bail!("YouTube is not configured.");
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_| anyhow::anyhow!("Unable to initialize the YouTube client."))?;
    let max_results = SEARCH_CANDIDATE_LIMIT.to_string();
    let search_response = client
        .get(SEARCH_ENDPOINT)
        .query(&[
            ("part", "snippet"),
            ("type", "video"),
            ("videoEmbeddable", "true"),
            ("maxResults", max_results.as_str()),
            ("order", "viewCount"),
            ("q", query.as_str()),
            ("key", api_key),
        ])
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("YouTube search failed."))?;
    if !search_response.status().is_success() {
        bail!(
            "YouTube search failed with status {}.",
            search_response.status()
        );
    }
    let search_response = serde_json::from_slice::<SearchResponse>(
        &search_response
            .bytes()
            .await
            .map_err(|_| anyhow::anyhow!("YouTube search returned an invalid response."))?,
    )
    .map_err(|_| anyhow::anyhow!("YouTube search returned an invalid response."))?;

    let search_items = search_response
        .items
        .into_iter()
        .filter_map(|item| item.id.video_id.map(|video_id| (video_id, item.snippet)))
        .collect::<Vec<_>>();
    if search_items.is_empty() {
        return Ok(Vec::new());
    }

    let video_ids = search_items
        .iter()
        .map(|(video_id, _)| video_id.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let videos_response = client
        .get(VIDEOS_ENDPOINT)
        .query(&[
            ("part", "contentDetails"),
            ("id", video_ids.as_str()),
            ("key", api_key),
        ])
        .send()
        .await
        .map_err(|_| anyhow::anyhow!("YouTube duration lookup failed."))?;
    if !videos_response.status().is_success() {
        bail!(
            "YouTube duration lookup failed with status {}.",
            videos_response.status()
        );
    }
    let videos_response =
        serde_json::from_slice::<VideoResponse>(&videos_response.bytes().await.map_err(|_| {
            anyhow::anyhow!("YouTube duration lookup returned an invalid response.")
        })?)
        .map_err(|_| anyhow::anyhow!("YouTube duration lookup returned an invalid response."))?;

    Ok(map_search_results(search_items, videos_response))
}

fn normalize_query(query: &str) -> Result<String> {
    let query = query.trim();
    if query.is_empty() || query.chars().any(char::is_control) {
        bail!("The YouTube query is invalid.");
    }
    Ok(query.chars().take(MAX_QUERY_CHARS).collect())
}

fn map_search_results(
    search_items: Vec<(String, VideoSnippet)>,
    videos_response: VideoResponse,
) -> Vec<YouTubeTrack> {
    let durations = videos_response
        .items
        .into_iter()
        .filter_map(|item| {
            parse_iso8601_duration(&item.content_details.duration)
                .map(|duration| (item.id, duration))
        })
        .collect::<HashMap<_, _>>();

    search_items
        .into_iter()
        .filter_map(|(video_id, snippet)| {
            let duration_seconds = durations.get(&video_id).copied()?;
            if duration_seconds == 0 || duration_seconds > MAX_TRACK_DURATION_SECONDS {
                return None;
            }
            let title = clean_text(&snippet.title, 200);
            let channel_title = clean_text(&snippet.channel_title, 100);
            let thumbnail = snippet
                .thumbnails
                .high
                .or(snippet.thumbnails.medium)
                .or(snippet.thumbnails.default)
                .map(|thumbnail| thumbnail.url)
                .filter(|url| url.starts_with("https://"))?;
            if title.is_empty() || channel_title.is_empty() {
                return None;
            }
            Some(YouTubeTrack {
                video_id,
                title,
                channel_title,
                thumbnail,
                duration_seconds,
            })
        })
        .take(RESULT_LIMIT)
        .collect()
}

fn clean_text(value: &str, max_chars: usize) -> String {
    value
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max_chars)
        .collect()
}

fn parse_iso8601_duration(value: &str) -> Option<u64> {
    let value = value.strip_prefix("PT")?;
    if value.is_empty() {
        return None;
    }

    let mut number = 0_u64;
    let mut total = 0_u64;
    let mut has_component = false;
    for character in value.chars() {
        if let Some(digit) = character.to_digit(10) {
            number = number.checked_mul(10)?.checked_add(u64::from(digit))?;
            continue;
        }
        let multiplier = match character {
            'H' => 3_600,
            'M' => 60,
            'S' => 1,
            _ => return None,
        };
        total = total.checked_add(number.checked_mul(multiplier)?)?;
        number = 0;
        has_component = true;
    }
    (number == 0 && has_component).then_some(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_youtube_durations() {
        assert_eq!(parse_iso8601_duration("PT2M57S"), Some(177));
        assert_eq!(parse_iso8601_duration("PT1H2M3S"), Some(3_723));
        assert_eq!(parse_iso8601_duration("PT0S"), Some(0));
        assert_eq!(parse_iso8601_duration("P2D"), None);
        assert_eq!(parse_iso8601_duration("PT2M30"), None);
    }

    #[test]
    fn normalizes_queries_without_allowing_controls() {
        assert_eq!(normalize_query("  daft   punk  ").unwrap(), "daft   punk");
        assert!(normalize_query("song\nname").is_err());
    }
}
