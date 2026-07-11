use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use rand::{RngCore, rng};
use windows::{
    Win32::{
        Foundation::VARIANT_FALSE,
        Media::Speech::{
            ISpeechFileStream, ISpeechObjectToken, ISpeechVoice, SSFMCreateForWrite, SVSFDefault,
            SpFileStream, SpVoice,
        },
        System::Com::{
            CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
        },
    },
    core::{BSTR, IUnknown},
};

pub struct SynthesizedSpeech {
    pub content_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpeechLanguage {
    Default,
    English,
    French,
}

pub async fn synthesize(text: String) -> Result<SynthesizedSpeech> {
    tauri::async_runtime::spawn_blocking(move || synthesize_blocking(&text))
        .await
        .context("Windows TTS worker failed")?
}

fn synthesize_blocking(text: &str) -> Result<SynthesizedSpeech> {
    if text.trim().is_empty() {
        bail!("TTS text is empty");
    }

    let _apartment = ComApartment::initialize()?;
    let voice_selector: ISpeechVoice = unsafe {
        CoCreateInstance(&SpVoice, None::<&IUnknown>, CLSCTX_ALL)
            .context("Windows has no available desktop speech voice")?
    };
    let language = detect_language(text);
    let voices = installed_voices(&voice_selector)?;
    let mut last_error = None;

    if language == SpeechLanguage::Default {
        match synthesize_with_voice(text, None) {
            Ok(bytes) => return synthesized_wave(bytes),
            Err(error) => last_error = Some(error),
        }
    } else {
        let matching_voices = preferred_voices(&voices, language);
        if matching_voices.is_empty() {
            bail!("Windows has no installed {language:?} speech voice");
        }
        for token in matching_voices {
            match synthesize_with_voice(text, Some(token)) {
                Ok(bytes) => return synthesized_wave(bytes),
                Err(error) => last_error = Some(error),
            }
        }
        return Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("Windows has no usable {language:?} speech voice")
        }));
    }

    for token in &voices {
        match synthesize_with_voice(text, Some(token)) {
            Ok(bytes) => return synthesized_wave(bytes),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Windows has no usable speech voice")))
}

fn installed_voices(voice_selector: &ISpeechVoice) -> Result<Vec<ISpeechObjectToken>> {
    let empty = BSTR::new();
    let voices = unsafe { voice_selector.GetVoices(&empty, &empty) }
        .context("Windows could not enumerate installed speech voices")?;
    let voice_count = unsafe { voices.Count() }.context("Windows could not count speech voices")?;
    (0..voice_count)
        .map(|index| {
            unsafe { voices.Item(index) }
                .with_context(|| format!("Windows could not access speech voice {index}"))
        })
        .collect()
}

fn preferred_voices(
    voices: &[ISpeechObjectToken],
    language: SpeechLanguage,
) -> Vec<&ISpeechObjectToken> {
    let mut matching = voices
        .iter()
        .filter(|token| token_language(token) == Some(language))
        .collect::<Vec<_>>();
    if language == SpeechLanguage::French {
        matching.sort_by_key(|token| {
            !token_description(token)
                .is_some_and(|description| description.to_lowercase().contains("hortense"))
        });
    }
    matching
}

fn token_language(token: &ISpeechObjectToken) -> Option<SpeechLanguage> {
    let attribute = unsafe { token.GetAttribute(&BSTR::from("Language")) }.ok()?;
    attribute
        .to_string()
        .split(';')
        .find_map(|language| u32::from_str_radix(language.trim(), 16).ok())
        .map(|language| match language & 0x03ff {
            0x0009 => SpeechLanguage::English,
            0x000c => SpeechLanguage::French,
            _ => SpeechLanguage::Default,
        })
}

fn token_description(token: &ISpeechObjectToken) -> Option<String> {
    unsafe { token.GetDescription(0) }
        .ok()
        .map(|description| description.to_string())
}

fn detect_language(text: &str) -> SpeechLanguage {
    let mut french_score = text
        .chars()
        .filter(|character| {
            matches!(
                character.to_ascii_lowercase(),
                'à' | 'â'
                    | 'æ'
                    | 'ç'
                    | 'é'
                    | 'è'
                    | 'ê'
                    | 'ë'
                    | 'î'
                    | 'ï'
                    | 'ô'
                    | 'œ'
                    | 'ù'
                    | 'û'
                    | 'ü'
                    | 'ÿ'
            )
        })
        .count()
        * 3;
    let mut english_score = 0;
    let normalized = text
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphabetic() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>();

    for word in normalized.split_whitespace() {
        french_score += language_word_score(word, FRENCH_STRONG_WORDS, FRENCH_COMMON_WORDS);
        english_score += language_word_score(word, ENGLISH_STRONG_WORDS, ENGLISH_COMMON_WORDS);
    }

    match french_score.cmp(&english_score) {
        std::cmp::Ordering::Greater => SpeechLanguage::French,
        std::cmp::Ordering::Less => SpeechLanguage::English,
        std::cmp::Ordering::Equal => SpeechLanguage::Default,
    }
}

fn language_word_score(word: &str, strong_words: &[&str], common_words: &[&str]) -> usize {
    if strong_words.contains(&word) {
        2
    } else if common_words.contains(&word) {
        1
    } else {
        0
    }
}

const FRENCH_STRONG_WORDS: &[&str] = &[
    "alors",
    "avec",
    "bonjour",
    "ça",
    "ceci",
    "cela",
    "cette",
    "comment",
    "dans",
    "des",
    "encore",
    "être",
    "français",
    "ici",
    "jamais",
    "leurs",
    "mais",
    "merci",
    "notre",
    "parce",
    "peux",
    "pourquoi",
    "salut",
    "très",
    "votre",
    "veux",
];
const FRENCH_COMMON_WORDS: &[&str] = &[
    "ai", "au", "aux", "car", "ce", "ces", "de", "dire", "du", "elle", "elles", "en", "est", "et",
    "faire", "fait", "il", "ils", "je", "la", "le", "les", "ma", "me", "mes", "mon", "ne", "non",
    "nous", "oui", "pas", "plus", "que", "qui", "se", "sont", "sur", "ta", "te", "tes", "ton",
    "tu", "un", "une", "va", "vais", "vous",
];
const ENGLISH_STRONG_WORDS: &[&str] = &[
    "again",
    "because",
    "could",
    "english",
    "hello",
    "here",
    "how",
    "something",
    "thanks",
    "that",
    "their",
    "there",
    "these",
    "this",
    "those",
    "very",
    "want",
    "well",
    "what",
    "when",
    "where",
    "which",
    "why",
    "with",
    "would",
    "your",
];
const ENGLISH_COMMON_WORDS: &[&str] = &[
    "am", "an", "and", "are", "as", "at", "be", "bro", "but", "can", "do", "for", "from", "have",
    "he", "i", "in", "is", "it", "its", "my", "no", "not", "of", "say", "she", "so", "some",
    "thank", "the", "they", "to", "we", "will", "yes", "you",
];

fn synthesize_with_voice(text: &str, token: Option<&ISpeechObjectToken>) -> Result<Vec<u8>> {
    let temporary_file = TemporaryWaveFile::new();
    let filename = BSTR::from(temporary_file.path.to_string_lossy().as_ref());
    let voice: ISpeechVoice = unsafe {
        CoCreateInstance(&SpVoice, None::<&IUnknown>, CLSCTX_ALL)
            .context("Windows could not create a speech voice")?
    };
    if let Some(token) = token {
        unsafe { voice.putref_Voice(token) }
            .context("Windows could not select an installed speech voice")?;
    }
    let stream: ISpeechFileStream = unsafe {
        CoCreateInstance(&SpFileStream, None::<&IUnknown>, CLSCTX_ALL)
            .context("Windows could not create a speech audio stream")?
    };
    unsafe {
        stream
            .Open(&filename, SSFMCreateForWrite, VARIANT_FALSE)
            .context("Windows could not open the temporary speech stream")?;
    }
    let speech_result = unsafe {
        voice
            .putref_AudioOutputStream(&stream)
            .and_then(|_| voice.Speak(&BSTR::from(text), SVSFDefault))
    };
    let close_result = unsafe { stream.Close() };
    speech_result.context("Windows could not synthesize the message")?;
    close_result.context("Windows could not finalize the speech stream")?;
    drop(stream);
    drop(voice);

    fs::read(&temporary_file.path).context("failed to read synthesized speech")
}

fn synthesized_wave(bytes: Vec<u8>) -> Result<SynthesizedSpeech> {
    if !bytes.starts_with(b"RIFF") {
        bail!("Windows TTS returned an invalid WAV stream");
    }
    Ok(SynthesizedSpeech {
        content_type: "audio/wav".into(),
        bytes,
    })
}

struct ComApartment;

impl ComApartment {
    fn initialize() -> Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .context("failed to initialize Windows COM for TTS")?;
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

struct TemporaryWaveFile {
    path: PathBuf,
}

impl TemporaryWaveFile {
    fn new() -> Self {
        let mut random = [0_u8; 16];
        rng().fill_bytes(&mut random);
        let suffix = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Self {
            path: std::env::temp_dir().join(format!("relay-tts-{suffix}.wav")),
        }
    }
}

impl Drop for TemporaryWaveFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_french_messages() {
        assert_eq!(
            detect_language("Bonjour, je veux te dire quelque chose de très important."),
            SpeechLanguage::French
        );
        assert_eq!(
            detect_language("C'était génial, merci beaucoup !"),
            SpeechLanguage::French
        );
        assert_eq!(
            detect_language("salut mon petit cochon"),
            SpeechLanguage::French
        );
    }

    #[test]
    fn detects_english_messages() {
        assert_eq!(
            detect_language("Hello, I want to tell you something very important."),
            SpeechLanguage::English
        );
        assert_eq!(
            detect_language("Thanks bro, this is really well done."),
            SpeechLanguage::English
        );
    }

    #[test]
    fn falls_back_for_ambiguous_messages() {
        assert_eq!(detect_language("OBS 1234"), SpeechLanguage::Default);
        assert_eq!(detect_language("merci hello"), SpeechLanguage::Default);
    }

    #[test]
    fn selects_dominant_language_in_mixed_messages() {
        assert_eq!(
            detect_language("Bonjour et merci, this is fine."),
            SpeechLanguage::French
        );
        assert_eq!(
            detect_language("Hello and thanks, je vais bien."),
            SpeechLanguage::English
        );
    }

    #[tokio::test]
    #[ignore = "requires fully installed French and English Windows speech packs"]
    async fn synthesizes_detected_french_and_english_messages() {
        for text in [
            "Bonjour, ceci est un test du relais français.",
            "Hello, this is an English relay test.",
        ] {
            let speech = synthesize(text.into()).await.unwrap();
            assert!(speech.bytes.starts_with(b"RIFF"));
            assert_eq!(speech.content_type, "audio/wav");
        }
    }

    #[test]
    #[ignore = "requires the optional Microsoft French language pack"]
    fn finds_hortense_in_installed_desktop_voices() {
        let _apartment = ComApartment::initialize().unwrap();
        let selector: ISpeechVoice =
            unsafe { CoCreateInstance(&SpVoice, None::<&IUnknown>, CLSCTX_ALL).unwrap() };
        let voices = installed_voices(&selector).unwrap();
        let french_voice = preferred_voices(&voices, SpeechLanguage::French)[0];
        assert!(
            token_description(french_voice)
                .unwrap()
                .to_lowercase()
                .contains("hortense")
        );
    }
}
