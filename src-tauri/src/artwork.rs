use std::{io::Cursor, time::Duration};

use anyhow::{Context, Result};
use futures_util::StreamExt;
use lofty::{file::TaggedFileExt, picture::PictureType, prelude::Accessor, probe::Probe};

pub const MAX_AUDIO_BYTES: usize = 50 * 1024 * 1024;
pub const MAX_ARTWORK_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_EMBED_MEDIA_BYTES: usize = 20 * 1024 * 1024;

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
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .build()?;
    let response = client.get(url).send().await?.error_for_status()?;
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
