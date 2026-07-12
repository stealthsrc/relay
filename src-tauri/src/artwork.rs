use std::{io::Cursor, sync::OnceLock, time::Duration};

use anyhow::{Context, Result};
use futures_util::StreamExt;
use lofty::{file::TaggedFileExt, picture::PictureType, prelude::Accessor, probe::Probe};

pub const MAX_AUDIO_BYTES: usize = 50 * 1024 * 1024;
pub const MAX_ARTWORK_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_EMBED_MEDIA_BYTES: usize = 20 * 1024 * 1024;

/// CDNs Discord serves media and embeds from. Downloads (including every
/// redirect hop) are restricted to these hosts to keep user-posted URLs from
/// steering requests at local or internal services.
const ALLOWED_HOST_SUFFIXES: &[&str] = &[
    "discordapp.com",
    "discordapp.net",
    "discord.com",
    "tenor.com",
    "tenor.co",
    "giphy.com",
    "imgur.com",
    "klipy.com",
];

fn url_allowed(url: &reqwest::Url) -> bool {
    if url.scheme() != "https" {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    ALLOWED_HOST_SUFFIXES
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}")))
}

fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 5 {
                    attempt.error("too many redirects")
                } else if url_allowed(attempt.url()) {
                    attempt.follow()
                } else {
                    attempt.error("redirect outside the allowed media hosts")
                }
            }))
            .build()
            .expect("valid HTTP client configuration")
    })
}

pub struct EmbeddedArtwork {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct AudioMetadata {
    pub audio: Vec<u8>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub artwork: Option<EmbeddedArtwork>,
}

pub async fn extract(url: &str) -> Result<AudioMetadata> {
    let audio = download_bounded(url, MAX_AUDIO_BYTES).await?;

    tauri::async_runtime::spawn_blocking(move || extract_from_bytes(audio))
        .await
        .context("audio artwork worker failed")?
}

pub async fn download_bounded(url: &str, maximum_bytes: usize) -> Result<Vec<u8>> {
    let parsed = reqwest::Url::parse(url).context("invalid media URL")?;
    if !url_allowed(&parsed) {
        anyhow::bail!("the media URL host is not allowed");
    }
    let response = http_client().get(parsed).send().await?.error_for_status()?;
    if response
        .content_length()
        .is_some_and(|length| length > maximum_bytes as u64)
    {
        anyhow::bail!("remote media exceeds the in-memory limit");
    }

    let mut audio = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if audio.len() + chunk.len() > maximum_bytes {
            anyhow::bail!("remote media exceeds the in-memory limit");
        }
        audio.extend_from_slice(&chunk);
    }

    Ok(audio)
}

fn extract_from_bytes(audio: Vec<u8>) -> Result<AudioMetadata> {
    let mut cursor = Cursor::new(&audio);
    let tagged = Probe::new(&mut cursor)
        .guess_file_type()
        .context("failed to identify the audio format")?
        .read()
        .context("failed to read audio metadata")?;
    let tag = tagged.primary_tag().or_else(|| tagged.first_tag());
    let title = tag
        .and_then(|tag| tag.title())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let artist = tag
        .and_then(|tag| tag.artist())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let pictures = tagged
        .tags()
        .iter()
        .flat_map(|tag| tag.pictures().iter())
        .collect::<Vec<_>>();
    let picture = pictures
        .iter()
        .copied()
        .find(|picture| picture.pic_type() == PictureType::CoverFront)
        .or_else(|| pictures.first().copied());
    let artwork = picture.and_then(|picture| {
        let bytes = picture.data();
        if bytes.is_empty() || bytes.len() > MAX_ARTWORK_BYTES {
            return None;
        }
        let content_type = picture.mime_type()?.as_str();
        if !matches!(
            content_type,
            "image/jpeg" | "image/png" | "image/gif" | "image/webp" | "image/bmp"
        ) {
            return None;
        }
        Some(EmbeddedArtwork {
            content_type: content_type.into(),
            bytes: bytes.to_vec(),
        })
    });
    Ok(AudioMetadata {
        audio,
        title,
        artist,
        artwork,
    })
}
