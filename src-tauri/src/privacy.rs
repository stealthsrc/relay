use std::{
    collections::{HashSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    io::Cursor,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::{Arc, OnceLock},
    time::Duration,
};

use anyhow::{Context, Result};
use exif::{In, Reader, Tag};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

pub const MAX_IMAGE_BYTES: usize = 12 * 1024 * 1024;
pub const MAX_IMAGE_DIMENSION: u32 = 16_384;
pub const MAX_IMAGE_PIXELS: u64 = 40_000_000;
pub const MAX_IMAGE_FRAMES: u32 = 256;
pub const MAX_ANIMATED_PIXELS: u64 = 500_000_000;
pub const OCR_TEXT_LIMIT: usize = 8_192;
pub const PRIVACY_TEXT_LIMIT: usize = 4_096;
const IMAGE_SCAN_TIMEOUT: Duration = Duration::from_secs(12);
const IMAGE_SCAN_CONCURRENCY: usize = 2;
pub const MAX_CONFIGURED_REGEXES: usize = 100;
pub const MAX_PRIVACY_LIST_ENTRIES: usize = 100;
pub const MAX_PRIVACY_LIST_VALUE_CHARS: usize = 256;
const MAX_REGEXES_PER_CONCEPT: usize = 25;
const MAX_REGEX_PATTERN_BYTES: usize = 512;
const REGEX_SIZE_LIMIT: usize = 256 * 1024;
const REGEX_DFA_SIZE_LIMIT: usize = 256 * 1024;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SuspiciousPolicy {
    Allow,
    #[default]
    Review,
    Block,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtectionLevel {
    #[default]
    Balanced,
    Strict,
    Paranoid,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PrivacyCategory {
    Email,
    Phone,
    IpAddress,
    GpsLocation,
    PostalAddress,
    Financial,
    LicensePlate,
    SensitiveUrl,
    CustomPattern,
    ContentFilter,
    ImageMetadata,
    Ocr,
    Document,
    MediaSafety,
}

impl PrivacyCategory {
    pub const USER_CONFIGURABLE: [Self; 12] = [
        Self::Email,
        Self::Phone,
        Self::IpAddress,
        Self::GpsLocation,
        Self::PostalAddress,
        Self::Financial,
        Self::LicensePlate,
        Self::SensitiveUrl,
        Self::CustomPattern,
        Self::ImageMetadata,
        Self::Ocr,
        Self::Document,
    ];

    pub fn log_code(self) -> &'static str {
        match self {
            Self::Email => "EMAIL",
            Self::Phone => "PHONE",
            Self::IpAddress => "IP_ADDRESS",
            Self::GpsLocation => "GPS_LOCATION",
            Self::PostalAddress => "POSTAL_ADDRESS",
            Self::Financial => "FINANCIAL",
            Self::LicensePlate => "LICENSE_PLATE",
            Self::SensitiveUrl => "SENSITIVE_URL",
            Self::CustomPattern => "CUSTOM_PATTERN",
            Self::ContentFilter => "CONTENT_FILTER",
            Self::ImageMetadata => "IMAGE_METADATA",
            Self::Ocr => "OCR",
            Self::Document => "DOCUMENT",
            Self::MediaSafety => "MEDIA_SAFETY",
        }
    }
}

pub fn default_privacy_categories() -> Vec<PrivacyCategory> {
    PrivacyCategory::USER_CONFIGURABLE.to_vec()
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForbiddenConcept {
    pub canonical: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub regexes: Vec<String>,
}

impl ForbiddenConcept {
    pub fn validate(&self) -> Result<()> {
        if !self.canonical.chars().any(char::is_alphabetic) {
            anyhow::bail!("Forbidden concepts must contain alphabetic characters.");
        }
        let canonical = normalize_compact(&self.canonical);
        if !(3..=64).contains(&canonical.chars().count()) {
            anyhow::bail!("Forbidden concept canonical values must contain 3 to 64 letters.");
        }
        if self.aliases.len() > 50 {
            anyhow::bail!("Each forbidden concept may contain at most 50 aliases.");
        }
        for alias in &self.aliases {
            if !alias.chars().any(char::is_alphabetic) {
                anyhow::bail!("Forbidden concept aliases must contain alphabetic characters.");
            }
            let normalized = normalize_compact(alias);
            if !(3..=64).contains(&normalized.chars().count()) {
                anyhow::bail!("Forbidden concept aliases must contain 3 to 64 letters.");
            }
        }
        if self.regexes.len() > MAX_REGEXES_PER_CONCEPT {
            anyhow::bail!(
                "Each forbidden concept may contain at most {MAX_REGEXES_PER_CONCEPT} regular expressions."
            );
        }
        for pattern in &self.regexes {
            if pattern.is_empty() || pattern.len() > MAX_REGEX_PATTERN_BYTES {
                anyhow::bail!(
                    "Forbidden concept regular expressions must contain 1 to {MAX_REGEX_PATTERN_BYTES} bytes."
                );
            }
            let regex = compile_filter_regex(pattern).map_err(|_| {
                anyhow::anyhow!("Forbidden concept regular expressions are invalid.")
            })?;
            if regex.is_match("") {
                anyhow::bail!(
                    "Forbidden concept regular expressions must not match an empty value."
                );
            }
        }
        Ok(())
    }
}

fn compile_filter_regex(pattern: &str) -> Result<Regex> {
    RegexBuilder::new(pattern)
        .case_insensitive(true)
        .size_limit(REGEX_SIZE_LIMIT)
        .dfa_size_limit(REGEX_DFA_SIZE_LIMIT)
        .build()
        .context("Invalid filter regular expression")
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PrivacyClassification {
    #[default]
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl PrivacyClassification {
    #[allow(non_upper_case_globals)]
    pub const Suspicious: Self = Self::Medium;
    #[allow(non_upper_case_globals)]
    pub const Sensitive: Self = Self::High;

    fn rank(self) -> u8 {
        match self {
            Self::Safe => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    fn log_code(self) -> &'static str {
        match self {
            Self::Safe => "SAFE",
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PrivacyReport {
    pub classification: PrivacyClassification,
    pub score: u8,
    pub categories: Vec<PrivacyCategory>,
    /// Stable, minimized reason codes. Never put OCR/EXIF values here.
    pub reasons: Vec<&'static str>,
    /// Signature of the privacy configuration snapshot used for this report.
    /// It is process-local metadata and is never serialized to the UI.
    pub config_signature: Option<u64>,
}

impl PrivacyReport {
    pub fn safe() -> Self {
        Self::default()
    }

    pub fn suspicious(reason: &'static str) -> Self {
        Self {
            classification: PrivacyClassification::Medium,
            score: 35,
            categories: reason_category(reason).into_iter().collect(),
            reasons: vec![reason],
            config_signature: None,
        }
    }

    pub fn low(reason: &'static str) -> Self {
        Self {
            classification: PrivacyClassification::Low,
            score: 10,
            categories: reason_category(reason).into_iter().collect(),
            reasons: vec![reason],
            config_signature: None,
        }
    }

    #[allow(dead_code)]
    pub fn sensitive(reason: &'static str) -> Self {
        Self {
            classification: PrivacyClassification::High,
            score: 70,
            categories: reason_category(reason).into_iter().collect(),
            reasons: vec![reason],
            config_signature: None,
        }
    }

    pub fn merge(&mut self, other: Self) {
        let other_signature = other.config_signature;
        if other.classification.rank() > self.classification.rank() {
            self.classification = other.classification;
        }
        self.score = self.score.saturating_add(other.score).min(100);
        for category in other.categories {
            if !self.categories.contains(&category) {
                self.categories.push(category);
            }
        }
        for reason in other.reasons {
            if !self.reasons.contains(&reason) {
                self.reasons.push(reason);
            }
        }
        if self.config_signature.is_none() {
            self.config_signature = other_signature;
        }
    }

    pub fn without_filter_signals(mut self) -> Self {
        let has_filter_signal = self.categories.contains(&PrivacyCategory::ContentFilter)
            || self.reasons.iter().any(|reason| {
                matches!(
                    *reason,
                    "forbidden_concept"
                        | "forbidden_regex"
                        | "forbidden_similarity"
                        | "similarity_score"
                )
            });
        if !has_filter_signal {
            return self;
        }
        self.reasons.retain(|reason| {
            !matches!(
                *reason,
                "forbidden_concept"
                    | "forbidden_regex"
                    | "forbidden_similarity"
                    | "similarity_score"
            )
        });
        self.classification = if self.reasons.is_empty() {
            PrivacyClassification::Safe
        } else if self.reasons.iter().any(|reason| {
            matches!(
                *reason,
                "gps"
                    | "coordinates"
                    | "ip_address"
                    | "partial_address"
                    | "location_profile"
                    | "address_or_visual_context"
                    | "image_limits"
            )
        }) {
            PrivacyClassification::High
        } else {
            PrivacyClassification::Medium
        };
        self.categories
            .retain(|category| *category != PrivacyCategory::ContentFilter);
        self.score = match self.classification {
            PrivacyClassification::Safe => 0,
            PrivacyClassification::Low => 15,
            PrivacyClassification::Medium => 35,
            PrivacyClassification::High => 70,
            PrivacyClassification::Critical => 100,
        };
        self
    }

    pub fn primary_reason(&self) -> Option<&'static str> {
        self.reasons.first().copied()
    }

    pub fn apply_score_policy(&mut self, config: &AppConfig) {
        let scored = risk_for_score(self.score, config.privacy_protection_level);
        if scored.rank() > self.classification.rank() {
            self.classification = scored;
        }
    }

    fn add_signal(
        &mut self,
        category: PrivacyCategory,
        points: u8,
        minimum: PrivacyClassification,
        reason: &'static str,
        config: &AppConfig,
    ) {
        self.score = self.score.saturating_add(points).min(100);
        if !self.categories.contains(&category) {
            self.categories.push(category);
        }
        if !self.reasons.contains(&reason) {
            self.reasons.push(reason);
        }
        let scored = risk_for_score(self.score, config.privacy_protection_level);
        self.classification = if scored.rank() > minimum.rank() {
            scored
        } else if minimum.rank() > self.classification.rank() {
            minimum
        } else {
            self.classification
        };
    }
}

fn reason_category(reason: &str) -> Option<PrivacyCategory> {
    Some(match reason {
        "email" => PrivacyCategory::Email,
        "phone" => PrivacyCategory::Phone,
        "ip_address" => PrivacyCategory::IpAddress,
        "gps" | "coordinates" | "location_profile" => PrivacyCategory::GpsLocation,
        "postal_address" | "partial_address" | "address_or_visual_context" => {
            PrivacyCategory::PostalAddress
        }
        "iban" | "payment_card" => PrivacyCategory::Financial,
        "license_plate" => PrivacyCategory::LicensePlate,
        "sensitive_url" => PrivacyCategory::SensitiveUrl,
        "custom_pattern" => PrivacyCategory::CustomPattern,
        "forbidden_concept" | "forbidden_regex" | "forbidden_similarity" | "similarity_score" => {
            PrivacyCategory::ContentFilter
        }
        "exif_metadata" => PrivacyCategory::ImageMetadata,
        "document" => PrivacyCategory::Document,
        "image_limits" | "mime_mismatch" => PrivacyCategory::MediaSafety,
        "ocr_text" => PrivacyCategory::Ocr,
        _ => return None,
    })
}

fn risk_for_score(score: u8, level: ProtectionLevel) -> PrivacyClassification {
    let (low, medium, high, critical) = match level {
        ProtectionLevel::Balanced => (15, 30, 60, 90),
        ProtectionLevel::Strict => (10, 25, 50, 80),
        ProtectionLevel::Paranoid => (5, 20, 40, 70),
    };
    if score >= critical {
        PrivacyClassification::Critical
    } else if score >= high {
        PrivacyClassification::High
    } else if score >= medium {
        PrivacyClassification::Medium
    } else if score >= low {
        PrivacyClassification::Low
    } else {
        PrivacyClassification::Safe
    }
}

#[derive(Clone, Debug, Default)]
struct ImageSignals {
    gps: bool,
    exif_incomplete: bool,
    ocr_text: String,
    ocr_truncated: bool,
    dimensions_valid: bool,
    decoded_pixels: u64,
    frame_count: u32,
    ocr_available: bool,
}

/// Applies all text-only rules. The returned report contains only coarse reason
/// codes, so OCR and Discord text never leave this function.
pub fn classify_text(text: Option<&str>, config: &AppConfig) -> PrivacyReport {
    let config_signature = Some(config_signature(config));
    if !privacy_rules_enabled(config) {
        return PrivacyReport {
            config_signature,
            ..PrivacyReport::safe()
        };
    }
    let Some(text) = text.filter(|value| !value.trim().is_empty()) else {
        return PrivacyReport {
            config_signature,
            ..PrivacyReport::safe()
        };
    };
    let (text, text_truncated) = cap_text(text);
    let cleaned = strip_invisible_characters(text);
    let masked = mask_allowlisted_values(&cleaned, &config.privacy_allowlist);
    let normalized = normalize_words(&masked);
    let mut report = PrivacyReport {
        config_signature,
        ..PrivacyReport::safe()
    };
    let concept_matches = forbidden_concept_match(&masked, &normalized, &config.privacy_concepts);
    if concept_matches.regex {
        report.add_signal(
            PrivacyCategory::ContentFilter,
            100,
            PrivacyClassification::Critical,
            "forbidden_regex",
            config,
        );
    }

    if normalized.is_empty() {
        if text_truncated {
            report.merge(PrivacyReport::low("scan_incomplete"));
        }
        return report;
    }
    if concept_matches.exact {
        report.add_signal(
            PrivacyCategory::ContentFilter,
            100,
            PrivacyClassification::Critical,
            "forbidden_concept",
            config,
        );
    }
    if concept_matches.similarities > 0 {
        report.add_signal(
            PrivacyCategory::ContentFilter,
            config
                .privacy_similarity_boost
                .saturating_mul(concept_matches.similarities)
                .saturating_mul(20),
            PrivacyClassification::Medium,
            if config.privacy_similarity_boost >= 4 {
                "similarity_score"
            } else {
                "forbidden_similarity"
            },
            config,
        );
    }

    if !config.privacy_scan_enabled {
        if text_truncated {
            report.merge(PrivacyReport::suspicious("scan_incomplete"));
        }
        return report;
    }

    let rule_words = normalize_rule_words(&masked);
    let has_email = category_enabled(config, PrivacyCategory::Email) && contains_email(&masked);
    if has_email {
        report.add_signal(
            PrivacyCategory::Email,
            15,
            PrivacyClassification::Low,
            "email",
            config,
        );
    }
    let has_phone = category_enabled(config, PrivacyCategory::Phone) && contains_phone(&masked);
    if has_phone {
        report.add_signal(
            PrivacyCategory::Phone,
            30,
            PrivacyClassification::Medium,
            "phone",
            config,
        );
    }
    if category_enabled(config, PrivacyCategory::IpAddress) && contains_ip_address(&masked) {
        report.add_signal(
            PrivacyCategory::IpAddress,
            30,
            PrivacyClassification::Medium,
            "ip_address",
            config,
        );
    }
    if category_enabled(config, PrivacyCategory::GpsLocation)
        && coordinate_signal(&masked).is_some()
    {
        report.add_signal(
            PrivacyCategory::GpsLocation,
            70,
            PrivacyClassification::High,
            "coordinates",
            config,
        );
    }
    let address = if category_enabled(config, PrivacyCategory::PostalAddress) {
        postal_address_signal(&masked, &rule_words)
    } else {
        AddressSignal::None
    };
    match address {
        AddressSignal::Partial => report.add_signal(
            PrivacyCategory::PostalAddress,
            15,
            PrivacyClassification::Low,
            "partial_address",
            config,
        ),
        AddressSignal::Probable => report.add_signal(
            PrivacyCategory::PostalAddress,
            60,
            PrivacyClassification::High,
            "postal_address",
            config,
        ),
        AddressSignal::Full => report.add_signal(
            PrivacyCategory::PostalAddress,
            65,
            PrivacyClassification::High,
            "postal_address",
            config,
        ),
        AddressSignal::None => {}
    }
    if category_enabled(config, PrivacyCategory::Financial) && contains_valid_iban(&masked) {
        report.add_signal(
            PrivacyCategory::Financial,
            75,
            PrivacyClassification::High,
            "iban",
            config,
        );
    }
    if category_enabled(config, PrivacyCategory::Financial) && contains_valid_payment_card(&masked)
    {
        report.add_signal(
            PrivacyCategory::Financial,
            100,
            PrivacyClassification::Critical,
            "payment_card",
            config,
        );
    }
    if category_enabled(config, PrivacyCategory::LicensePlate) && contains_license_plate(&masked) {
        report.add_signal(
            PrivacyCategory::LicensePlate,
            35,
            PrivacyClassification::Medium,
            "license_plate",
            config,
        );
    }
    if category_enabled(config, PrivacyCategory::SensitiveUrl) && contains_sensitive_url(&masked) {
        report.add_signal(
            PrivacyCategory::SensitiveUrl,
            45,
            PrivacyClassification::Medium,
            "sensitive_url",
            config,
        );
    }
    if category_enabled(config, PrivacyCategory::Document) && contains_document_signal(&masked) {
        report.add_signal(
            PrivacyCategory::Document,
            50,
            PrivacyClassification::Medium,
            "document",
            config,
        );
    }
    if category_enabled(config, PrivacyCategory::CustomPattern)
        && custom_pattern_match(&normalized, &config.privacy_custom_patterns)
    {
        report.add_signal(
            PrivacyCategory::CustomPattern,
            75,
            PrivacyClassification::High,
            "custom_pattern",
            config,
        );
    }

    let has_name = contains_person_name_label(&rule_words);
    if has_name && matches!(address, AddressSignal::Probable | AddressSignal::Full) {
        report.add_signal(
            PrivacyCategory::PostalAddress,
            30,
            PrivacyClassification::High,
            "person_address_combination",
            config,
        );
    }
    if has_name && has_phone {
        report.add_signal(
            PrivacyCategory::Phone,
            35,
            PrivacyClassification::High,
            "person_phone_combination",
            config,
        );
    }
    if has_email && has_phone && !matches!(address, AddressSignal::None) {
        report.classification = PrivacyClassification::Critical;
        report.score = 100;
    }
    if text_truncated {
        report.add_signal(
            PrivacyCategory::MediaSafety,
            5,
            PrivacyClassification::Low,
            "scan_incomplete",
            config,
        );
    }
    report
}

/// Downloads and inspects an image from the Discord CDN. Failures are kept
/// deliberately coarse and become a reviewable signal rather than leaking
/// parser/network details to the UI or logs.
pub async fn analyze_remote_image(
    url: &str,
    proxy_url: &str,
    text: Option<&str>,
    config: &AppConfig,
) -> PrivacyReport {
    let mut report = classify_text(text, config);
    if !config.privacy_scan_enabled {
        return report;
    }

    let bytes = match download_image_bounded(url).await {
        Ok(bytes) => bytes,
        Err(_) if proxy_url != url => match download_image_bounded(proxy_url).await {
            Ok(bytes) => bytes,
            Err(_) => {
                report.merge(PrivacyReport::suspicious("image_fetch_unavailable"));
                return report;
            }
        },
        Err(_) => {
            report.merge(PrivacyReport::suspicious("image_fetch_unavailable"));
            return report;
        }
    };

    let scanned = analyze_image_bytes_async(&bytes, text, config).await;
    report.merge(scanned);
    report.apply_score_policy(config);
    report
}

pub fn image_limit_report(text: Option<&str>, config: &AppConfig) -> PrivacyReport {
    let mut report = classify_text(text, config);
    report.add_signal(
        PrivacyCategory::MediaSafety,
        80,
        PrivacyClassification::High,
        "image_limits",
        config,
    );
    report
}

/// Deterministic in-memory image analysis entry point used by the bot and
/// fixtures. It never returns image bytes, EXIF values, or OCR text.
pub fn analyze_image_bytes(bytes: &[u8], text: Option<&str>, config: &AppConfig) -> PrivacyReport {
    let mut report = classify_text(text, config);
    if !config.privacy_scan_enabled {
        return report;
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        report.add_signal(
            PrivacyCategory::MediaSafety,
            80,
            PrivacyClassification::High,
            "image_limits",
            config,
        );
        return report;
    }
    if !supported_image_signature(bytes) {
        report.add_signal(
            PrivacyCategory::MediaSafety,
            70,
            PrivacyClassification::High,
            "mime_mismatch",
            config,
        );
        return report;
    }
    // Parse GPS independently of the platform decoder so a malformed or
    // unsupported image cannot turn a location-bearing asset into a safe one.
    // EXIF parsing remains best-effort and never exposes field values.
    let exif = parse_exif(bytes);
    if exif.gps && category_enabled(config, PrivacyCategory::GpsLocation) {
        report.add_signal(
            PrivacyCategory::GpsLocation,
            75,
            PrivacyClassification::High,
            "gps",
            config,
        );
    }
    if exif.sensitive_metadata && category_enabled(config, PrivacyCategory::ImageMetadata) {
        report.add_signal(
            PrivacyCategory::ImageMetadata,
            10,
            PrivacyClassification::Low,
            "exif_metadata",
            config,
        );
    }
    let bytes = bytes.to_vec();
    let ocr_requested = category_enabled(config, PrivacyCategory::Ocr);
    let signals = match inspect_image(bytes, ocr_requested) {
        Ok(signals) => signals,
        Err(_) => {
            if exif.incomplete {
                report.merge(PrivacyReport::low("scan_incomplete"));
            }
            report.merge(PrivacyReport::low("image_scan_unavailable"));
            return report;
        }
    };
    if signals.gps && category_enabled(config, PrivacyCategory::GpsLocation) {
        report.add_signal(
            PrivacyCategory::GpsLocation,
            75,
            PrivacyClassification::High,
            "gps",
            config,
        );
    }
    if exif.incomplete || signals.exif_incomplete || signals.ocr_truncated {
        report.merge(PrivacyReport::low("scan_incomplete"));
    }
    if !signals.dimensions_valid
        || signals.frame_count == 0
        || signals.frame_count > MAX_IMAGE_FRAMES
        || signals
            .decoded_pixels
            .saturating_mul(u64::from(signals.frame_count))
            > MAX_ANIMATED_PIXELS
    {
        report.add_signal(
            PrivacyCategory::MediaSafety,
            80,
            PrivacyClassification::High,
            "image_limits",
            config,
        );
        return report;
    }
    if ocr_requested && signals.ocr_available {
        let ocr_report = classify_text(Some(&signals.ocr_text), config);
        if ocr_report.classification != PrivacyClassification::Safe
            && !report.categories.contains(&PrivacyCategory::Ocr)
        {
            report.categories.push(PrivacyCategory::Ocr);
        }
        report.merge(ocr_report);
    } else if ocr_requested {
        report.merge(PrivacyReport::low("scan_incomplete"));
    }
    report.apply_score_policy(config);
    report
}

/// Runs the platform decoder behind a bounded worker pool. The semaphore
/// permit is moved into the blocking closure, so a timed-out WinRT operation
/// cannot cause unbounded concurrent decoder jobs.
pub async fn analyze_image_bytes_async(
    bytes: &[u8],
    text: Option<&str>,
    config: &AppConfig,
) -> PrivacyReport {
    if !config.privacy_scan_enabled {
        return classify_text(text, config);
    }
    let semaphore = scan_semaphore();
    let Ok(Ok(permit)) =
        tokio::time::timeout(IMAGE_SCAN_TIMEOUT, semaphore.clone().acquire_owned()).await
    else {
        return PrivacyReport::low("scan_incomplete");
    };
    let bytes = bytes.to_vec();
    let text = text.map(str::to_owned);
    let config = config.clone();
    match tokio::time::timeout(
        IMAGE_SCAN_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            analyze_image_bytes(&bytes, text.as_deref(), &config)
        }),
    )
    .await
    {
        Ok(Ok(report)) => report,
        _ => PrivacyReport::low("scan_incomplete"),
    }
}

async fn download_image_bounded(url: &str) -> Result<Vec<u8>> {
    let parsed = reqwest::Url::parse(url).context("invalid image URL")?;
    if !discord_cdn_url(&parsed) {
        anyhow::bail!("image URL host is not an approved Discord CDN");
    }
    let response = image_client()
        .get(parsed)
        .send()
        .await?
        .error_for_status()?;
    if response
        .content_length()
        .is_some_and(|length| length > MAX_IMAGE_BYTES as u64)
    {
        anyhow::bail!("image exceeds the in-memory limit");
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if bytes.len() + chunk.len() > MAX_IMAGE_BYTES {
            anyhow::bail!("image exceeds the in-memory limit");
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn scan_semaphore() -> &'static Arc<tokio::sync::Semaphore> {
    static SEMAPHORE: OnceLock<Arc<tokio::sync::Semaphore>> = OnceLock::new();
    SEMAPHORE.get_or_init(|| Arc::new(tokio::sync::Semaphore::new(IMAGE_SCAN_CONCURRENCY)))
}

fn image_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 4 {
                    attempt.error("too many redirects")
                } else if discord_cdn_url(attempt.url()) {
                    attempt.follow()
                } else {
                    attempt.error("redirect outside the Discord CDN")
                }
            }))
            .build()
            .expect("valid privacy image client")
    })
}

fn discord_cdn_url(url: &reqwest::Url) -> bool {
    if url.scheme() != "https" || url.username() != "" || url.password().is_some() {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    ["discordapp.com", "discordapp.net", "discord.com"]
        .into_iter()
        .any(|suffix| host == suffix || host.ends_with(&format!(".{suffix}")))
}

fn supported_image_signature(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.starts_with(b"GIF87a")
        || bytes.starts_with(b"GIF89a")
        || bytes.starts_with(&[0xff, 0xd8])
        || (bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP".as_slice()))
        || bytes.starts_with(b"BM")
}

fn inspect_image(bytes: Vec<u8>, ocr_requested: bool) -> Result<ImageSignals> {
    let exif = parse_exif(&bytes);
    let gps = exif.gps;
    #[cfg(target_os = "windows")]
    {
        let mut signals = inspect_image_windows(&bytes, ocr_requested)?;
        signals.gps = gps;
        signals.exif_incomplete = exif.incomplete;
        Ok(signals)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let (width, height, frame_count) = basic_image_limits(&bytes);
        Ok(ImageSignals {
            gps,
            exif_incomplete: exif.incomplete,
            ocr_text: String::new(),
            ocr_truncated: false,
            dimensions_valid: width > 0
                && height > 0
                && width <= MAX_IMAGE_DIMENSION
                && height <= MAX_IMAGE_DIMENSION
                && u64::from(width) * u64::from(height) <= MAX_IMAGE_PIXELS,
            decoded_pixels: u64::from(width) * u64::from(height),
            frame_count,
            ocr_available: false,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ExifSignals {
    gps: bool,
    sensitive_metadata: bool,
    incomplete: bool,
}

fn parse_exif(bytes: &[u8]) -> ExifSignals {
    let mut cursor = Cursor::new(bytes);
    let exif = match Reader::new().read_from_container(&mut cursor) {
        Ok(exif) => exif,
        Err(exif::Error::NotFound(_)) => return ExifSignals::default(),
        Err(_) => {
            return ExifSignals {
                incomplete: true,
                ..ExifSignals::default()
            };
        }
    };
    let has_lat = exif.get_field(Tag::GPSLatitude, In::PRIMARY).is_some();
    let has_lon = exif.get_field(Tag::GPSLongitude, In::PRIMARY).is_some();
    let has_ref = exif
        .get_field(Tag::GPSLatitudeRef, In::PRIMARY)
        .or_else(|| exif.get_field(Tag::GPSLongitudeRef, In::PRIMARY))
        .is_some();
    let coordinate_pair = has_lat && has_lon;
    let coordinate_with_ref = has_ref && (has_lat || has_lon);
    let sensitive_metadata = [
        Tag::Make,
        Tag::Model,
        Tag::DateTime,
        Tag::DateTimeOriginal,
        Tag::BodySerialNumber,
        Tag::LensSerialNumber,
        Tag::UserComment,
    ]
    .into_iter()
    .any(|tag| exif.get_field(tag, In::PRIMARY).is_some());
    ExifSignals {
        gps: coordinate_pair || coordinate_with_ref,
        sensitive_metadata,
        incomplete: false,
    }
}

#[cfg(not(target_os = "windows"))]
fn basic_image_limits(bytes: &[u8]) -> (u32, u32, u32) {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.len() >= 24 {
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap_or_default());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap_or_default());
        return (width, height, 1);
    }
    if bytes.starts_with(b"GIF8") && bytes.len() >= 10 {
        let width = u16::from_le_bytes(bytes[6..8].try_into().unwrap_or_default()) as u32;
        let height = u16::from_le_bytes(bytes[8..10].try_into().unwrap_or_default()) as u32;
        let frames = bytes
            .windows(8)
            .filter(|window| *window == b"\x00\x21\xF9\x04")
            .count();
        return (width, height, frames.max(1) as u32);
    }
    if bytes.starts_with(&[0xff, 0xd8]) {
        let mut index = 2;
        while index + 9 < bytes.len() {
            if bytes[index] != 0xff {
                index += 1;
                continue;
            }
            let marker = bytes[index + 1];
            index += 2;
            if marker == 0xd8 || marker == 0xd9 || (0xd0..=0xd7).contains(&marker) {
                continue;
            }
            if index + 2 > bytes.len() {
                break;
            }
            let length = u16::from_be_bytes([bytes[index], bytes[index + 1]]) as usize;
            if length < 2 || index + length > bytes.len() {
                break;
            }
            if (0xc0..=0xc3).contains(&marker)
                || (0xc5..=0xc7).contains(&marker)
                || (0xc9..=0xcb).contains(&marker)
                || (0xcd..=0xcf).contains(&marker)
            {
                if length >= 7 {
                    let height = u16::from_be_bytes([bytes[index + 3], bytes[index + 4]]) as u32;
                    let width = u16::from_be_bytes([bytes[index + 5], bytes[index + 6]]) as u32;
                    return (width, height, 1);
                }
            }
            index += length;
        }
    }
    (0, 0, 0)
}

#[cfg(target_os = "windows")]
fn inspect_image_windows(bytes: &[u8], ocr_requested: bool) -> Result<ImageSignals> {
    use windows::{
        Globalization::Language,
        Graphics::Imaging::BitmapDecoder,
        Media::Ocr::OcrEngine,
        Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
        Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
        core::HSTRING,
    };

    unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.context("WinRT initialization failed")?;
    struct RoGuard;
    impl Drop for RoGuard {
        fn drop(&mut self) {
            unsafe { RoUninitialize() };
        }
    }
    let _ro_guard = RoGuard;
    (|| {
        let stream = InMemoryRandomAccessStream::new()?;
        let output = stream.GetOutputStreamAt(0)?;
        let writer = DataWriter::CreateDataWriter(&output)?;
        writer.WriteBytes(bytes)?;
        writer.StoreAsync()?.get()?;
        writer.FlushAsync()?.get()?;
        writer.DetachStream()?;
        stream.Seek(0)?;
        let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
        let frame_count = decoder.FrameCount()?;
        let frame = decoder.GetFrameAsync(0)?.get()?;
        let width = frame.PixelWidth()?;
        let height = frame.PixelHeight()?;
        let dimensions_valid = width > 0
            && height > 0
            && width <= MAX_IMAGE_DIMENSION
            && height <= MAX_IMAGE_DIMENSION
            && u64::from(width) * u64::from(height) <= MAX_IMAGE_PIXELS;
        if !dimensions_valid || frame_count == 0 || frame_count > MAX_IMAGE_FRAMES {
            return Ok(ImageSignals {
                gps: false,
                exif_incomplete: false,
                ocr_text: String::new(),
                ocr_truncated: false,
                dimensions_valid,
                decoded_pixels: u64::from(width) * u64::from(height),
                frame_count,
                ocr_available: false,
            });
        }
        let bitmap = if ocr_requested {
            Some(frame.GetSoftwareBitmapAsync()?.get()?)
        } else {
            None
        };
        let mut ocr_text = String::new();
        let mut ocr_truncated = false;
        let mut ocr_available = false;
        for tag in ["fr-FR", "en-US"] {
            let Some(bitmap) = bitmap.as_ref() else {
                break;
            };
            let Ok(language) = Language::CreateLanguage(&HSTRING::from(tag)) else {
                continue;
            };
            let Ok(engine) = OcrEngine::TryCreateFromLanguage(&language) else {
                continue;
            };
            let Ok(result) = engine
                .RecognizeAsync(bitmap)
                .and_then(|operation| operation.get())
            else {
                continue;
            };
            let Ok(text) = result.Text() else {
                continue;
            };
            let text = text.to_string();
            ocr_truncated = text.chars().count() > OCR_TEXT_LIMIT;
            ocr_text = text.chars().take(OCR_TEXT_LIMIT).collect();
            ocr_available = true;
            if !ocr_text.trim().is_empty() {
                break;
            }
        }
        Ok(ImageSignals {
            gps: false,
            exif_incomplete: false,
            ocr_text,
            ocr_truncated,
            dimensions_valid,
            decoded_pixels: u64::from(width) * u64::from(height),
            frame_count,
            ocr_available,
        })
    })()
}

fn category_enabled(config: &AppConfig, category: PrivacyCategory) -> bool {
    matches!(
        category,
        PrivacyCategory::ContentFilter | PrivacyCategory::MediaSafety
    ) || config.privacy_enabled_categories.contains(&category)
}

fn strip_invisible_characters(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            !matches!(
                *character,
                '\u{00ad}'
                    | '\u{034f}'
                    | '\u{061c}'
                    | '\u{180e}'
                    | '\u{200b}'..='\u{200f}'
                    | '\u{202a}'..='\u{202e}'
                    | '\u{2060}'..='\u{206f}'
                    | '\u{feff}'
            )
        })
        .collect()
}

fn mask_allowlisted_values(text: &str, allowlist: &[String]) -> String {
    let mut masked = text.to_owned();
    for value in allowlist {
        let value = value.trim();
        if value.chars().count() < 3 {
            continue;
        }
        let Ok(regex) = RegexBuilder::new(&regex::escape(value))
            .case_insensitive(true)
            .size_limit(REGEX_SIZE_LIMIT)
            .dfa_size_limit(REGEX_DFA_SIZE_LIMIT)
            .build()
        else {
            continue;
        };
        masked = regex.replace_all(&masked, " ").into_owned();
    }
    masked
}

fn deobfuscate_contact_text(text: &str) -> String {
    static AT: OnceLock<Regex> = OnceLock::new();
    static DOT: OnceLock<Regex> = OnceLock::new();
    let at = AT.get_or_init(|| {
        Regex::new(r"(?iu)\s*(?:\[\s*at\s*\]|\(\s*at\s*\)|\bat\b)\s*")
            .expect("valid email at matcher")
    });
    let dot = DOT.get_or_init(|| {
        Regex::new(r"(?iu)\s*(?:\[\s*dot\s*\]|\(\s*dot\s*\)|\bdot\b)\s*")
            .expect("valid email dot matcher")
    });
    let text = at.replace_all(text, "@");
    dot.replace_all(&text, ".").into_owned()
}

fn contains_email(text: &str) -> bool {
    static EMAIL: OnceLock<Regex> = OnceLock::new();
    let email = EMAIL.get_or_init(|| {
        Regex::new(
            r"(?iu)\b[a-z0-9.!#$%&'*+/=?^_`{|}~-]{1,64}@[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)+\b",
        )
        .expect("valid email matcher")
    });
    email.is_match(&deobfuscate_contact_text(text))
}

fn numeric_candidates(text: &str) -> Vec<(String, String)> {
    let mut candidates = Vec::new();
    let mut raw = String::new();
    let mut digits = String::new();
    let mut started = false;
    let flush = |raw: &mut String, digits: &mut String, candidates: &mut Vec<(String, String)>| {
        if !digits.is_empty() {
            candidates.push((std::mem::take(raw), std::mem::take(digits)));
        } else {
            raw.clear();
        }
    };
    for character in text.chars().chain(std::iter::once('\0')) {
        if character.is_ascii_digit() {
            raw.push(character);
            digits.push(character);
            started = true;
        } else if started && matches!(character, '+' | '-' | '.' | '/' | '(' | ')' | ' ') {
            raw.push(character);
        } else if !started && character == '+' {
            raw.push(character);
            started = true;
        } else {
            flush(&mut raw, &mut digits, &mut candidates);
            started = false;
        }
    }
    candidates
}

fn contains_phone(text: &str) -> bool {
    numeric_candidates(text).into_iter().any(|(raw, digits)| {
        if !(10..=15).contains(&digits.len()) {
            return false;
        }
        let separators = raw
            .chars()
            .filter(|character| matches!(character, ' ' | '-' | '.' | '/' | '(' | ')'))
            .count();
        let international = raw.trim_start().starts_with('+') || digits.starts_with("00");
        let national = digits.len() == 10 && digits.starts_with('0');
        (international || national || (digits.len() == 10 && separators >= 2))
            && !looks_like_payment_card(&digits)
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AddressSignal {
    None,
    Partial,
    Probable,
    Full,
}

fn postal_address_signal(text: &str, words: &[String]) -> AddressSignal {
    let street_types = [
        "allee",
        "avenue",
        "av",
        "boulevard",
        "bd",
        "chemin",
        "drive",
        "impasse",
        "lane",
        "place",
        "quai",
        "road",
        "route",
        "rue",
        "square",
        "street",
        "voie",
        "way",
    ];
    for (index, word) in words.iter().enumerate() {
        if !(1..=5).contains(&word.len())
            || !word.chars().all(|character| character.is_ascii_digit())
        {
            continue;
        }
        let following = words.iter().skip(index + 1).take(5).collect::<Vec<_>>();
        let Some(type_index) = following
            .iter()
            .position(|candidate| street_types.contains(&candidate.as_str()))
        else {
            if has_address_prefix(text)
                && following.first().is_some_and(|candidate| {
                    candidate.chars().count() >= 3 && candidate.chars().all(char::is_alphabetic)
                })
            {
                return AddressSignal::Probable;
            }
            continue;
        };
        let has_street_name = following
            .iter()
            .skip(type_index + 1)
            .take(3)
            .any(|candidate| {
                candidate.chars().count() >= 2 && candidate.chars().all(char::is_alphabetic)
            })
            || (type_index > 0
                && following[..type_index].iter().any(|candidate| {
                    candidate.chars().count() >= 2 && candidate.chars().all(char::is_alphabetic)
                }));
        if !has_street_name {
            return AddressSignal::Partial;
        }
        let tail = words.iter().skip(index + 1).take(12).collect::<Vec<_>>();
        let has_postcode_city = tail.windows(2).any(|pair| {
            (4..=6).contains(&pair[0].len())
                && pair[0].chars().all(|character| character.is_ascii_digit())
                && pair[1].chars().count() >= 2
                && pair[1].chars().all(char::is_alphabetic)
        });
        return if has_postcode_city {
            AddressSignal::Full
        } else {
            AddressSignal::Probable
        };
    }
    AddressSignal::None
}

fn contains_valid_iban(text: &str) -> bool {
    static IBAN: OnceLock<Regex> = OnceLock::new();
    let regex = IBAN.get_or_init(|| {
        Regex::new(r"(?iu)\b[A-Z]{2}\s*\d{2}(?:[\s-]?[A-Z0-9]){11,30}\b")
            .expect("valid IBAN matcher")
    });
    regex.find_iter(text).any(|candidate| {
        let mut compact = candidate
            .as_str()
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .map(|character| character.to_ascii_uppercase())
            .collect::<String>();
        if let Some(expected) = iban_country_length(&compact[..compact.len().min(2)])
            && compact.len() >= expected
        {
            compact.truncate(expected);
        }
        if !(15..=34).contains(&compact.len())
            || !compact[..2]
                .chars()
                .all(|character| character.is_ascii_alphabetic())
            || !compact[2..4]
                .chars()
                .all(|character| character.is_ascii_digit())
        {
            return false;
        }
        let rearranged = format!("{}{}", &compact[4..], &compact[..4]);
        let mut remainder = 0_u32;
        for character in rearranged.chars() {
            if character.is_ascii_digit() {
                remainder = (remainder * 10 + character.to_digit(10).unwrap_or_default()) % 97;
            } else {
                let value = character as u32 - 'A' as u32 + 10;
                remainder = (remainder * 100 + value) % 97;
            }
        }
        remainder == 1
    })
}

fn iban_country_length(country: &str) -> Option<usize> {
    Some(match country {
        "AT" => 20,
        "BE" => 16,
        "CH" => 21,
        "DE" => 22,
        "DK" | "FI" | "NO" => 18,
        "ES" | "PT" => 24,
        "FR" | "IT" => 27,
        "GB" => 22,
        "IE" => 22,
        "LU" => 20,
        "NL" => 18,
        "PL" => 28,
        "SE" => 24,
        _ => return None,
    })
}

fn contains_valid_payment_card(text: &str) -> bool {
    numeric_candidates(text)
        .into_iter()
        .any(|(_, digits)| looks_like_payment_card(&digits) && luhn_valid(&digits))
}

fn looks_like_payment_card(digits: &str) -> bool {
    let length = digits.len();
    let prefix2 = digits.get(..2).and_then(|value| value.parse::<u16>().ok());
    let prefix4 = digits.get(..4).and_then(|value| value.parse::<u16>().ok());
    (digits.starts_with('4') && matches!(length, 13 | 16 | 19))
        || (length == 16
            && (prefix2.is_some_and(|value| (51..=55).contains(&value))
                || prefix4.is_some_and(|value| (2221..=2720).contains(&value))
                || digits.starts_with("6011")
                || prefix2.is_some_and(|value| (64..=65).contains(&value))))
        || (length == 15 && matches!(prefix2, Some(34 | 37)))
}

fn luhn_valid(digits: &str) -> bool {
    if digits.bytes().all(|digit| digit == digits.as_bytes()[0]) {
        return false;
    }
    let sum = digits
        .bytes()
        .rev()
        .enumerate()
        .map(|(index, digit)| {
            let mut value = u32::from(digit - b'0');
            if index % 2 == 1 {
                value *= 2;
                if value > 9 {
                    value -= 9;
                }
            }
            value
        })
        .sum::<u32>();
    sum % 10 == 0
}

fn contains_license_plate(text: &str) -> bool {
    static PLATE: OnceLock<Regex> = OnceLock::new();
    let regex = PLATE.get_or_init(|| {
        Regex::new(r"(?iu)\b[A-Z]{2}[- ]?\d{3}[- ]?[A-Z]{2}\b")
            .expect("valid license plate matcher")
    });
    let context = normalize_rule_words(text).iter().any(|word| {
        matches!(
            word.as_str(),
            "immat" | "immatriculation" | "license" | "plate" | "plaque" | "registration"
        )
    });
    regex
        .find_iter(text)
        .any(|candidate| context || candidate.as_str().contains(['-', ' ']))
}

fn contains_sensitive_url(text: &str) -> bool {
    static URL: OnceLock<Regex> = OnceLock::new();
    let regex = URL.get_or_init(|| {
        Regex::new(r#"(?iu)https?://[^\s<>\"']{1,2048}"#).expect("valid URL matcher")
    });
    regex.find_iter(text).any(|candidate| {
        let trimmed = candidate
            .as_str()
            .trim_end_matches([',', '.', ')', ']', '}', ';']);
        let Ok(url) = reqwest::Url::parse(trimmed) else {
            return false;
        };
        if !url.username().is_empty() || url.password().is_some() {
            return true;
        }
        url.query_pairs().any(|(key, value)| {
            matches!(
                normalize_compact(&key).as_str(),
                "access_token"
                    | "accesstoken"
                    | "address"
                    | "adresse"
                    | "apikey"
                    | "auth"
                    | "email"
                    | "iban"
                    | "key"
                    | "lat"
                    | "latitude"
                    | "lon"
                    | "longitude"
                    | "phone"
                    | "telephone"
                    | "token"
            ) || contains_email(&value)
                || contains_phone(&value)
                || contains_ip_address(&value)
        })
    })
}

fn contains_document_signal(text: &str) -> bool {
    let normalized = normalize_rule_words(text);
    let keyword_count = normalized
        .iter()
        .filter(|word| {
            matches!(
                word.as_str(),
                "administration"
                    | "birth"
                    | "carte"
                    | "date"
                    | "driver"
                    | "identity"
                    | "identite"
                    | "license"
                    | "naissance"
                    | "nationalite"
                    | "passport"
                    | "passeport"
                    | "permis"
                    | "securite"
                    | "social"
                    | "surname"
            )
        })
        .count();
    let long_number = numeric_candidates(text)
        .iter()
        .any(|(_, digits)| digits.len() >= 8);
    keyword_count >= 3 || (keyword_count >= 2 && long_number)
}

fn custom_pattern_match(words: &[String], patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| {
        let expected = normalize_compact(pattern);
        expected.chars().count() >= 3
            && matches!(
                variant_match(words, &expected),
                Some(ConceptMatchKind::Exact)
            )
    })
}

fn contains_person_name_label(words: &[String]) -> bool {
    words.iter().enumerate().any(|(index, word)| {
        matches!(
            word.as_str(),
            "fullname" | "name" | "nom" | "prenom" | "surname"
        ) && words
            .iter()
            .skip(index + 1)
            .take(3)
            .filter(|candidate| {
                candidate.chars().count() >= 2 && candidate.chars().all(char::is_alphabetic)
            })
            .count()
            >= 2
    })
}

#[allow(dead_code)]
fn address_score(words: &[String]) -> i32 {
    let street_words: HashSet<&str> = [
        "street",
        "st",
        "rue",
        "road",
        "rd",
        "avenue",
        "ave",
        "boulevard",
        "blvd",
        "drive",
        "dr",
        "lane",
        "ln",
        "route",
        "way",
        "place",
        "pl",
        "chemin",
        "allée",
    ]
    .into_iter()
    .collect();
    let number_indices = words
        .iter()
        .enumerate()
        .filter_map(|(index, word)| {
            (word.chars().all(|character| character.is_ascii_digit())
                && (1..=6).contains(&word.len()))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    let street_indices = words
        .iter()
        .enumerate()
        .filter_map(|(index, word)| street_words.contains(word.as_str()).then_some(index))
        .collect::<Vec<_>>();
    let has_number_before_street = street_indices.iter().any(|street_index| {
        number_indices
            .iter()
            .any(|number_index| number_index < street_index && street_index - number_index <= 3)
    });
    if !has_number_before_street {
        return 0;
    }
    let has_number = words.iter().any(|word| {
        word.chars().all(|character| character.is_ascii_digit()) && (1..=6).contains(&word.len())
    });
    let has_postal = words
        .iter()
        .any(|word| word.len() == 5 && word.chars().all(|character| character.is_ascii_digit()));
    if has_number {
        if has_postal { 6 } else { 4 }
    } else {
        0
    }
}

#[allow(dead_code)]
fn visual_context_score(words: &[String]) -> i32 {
    let document: &[&str] = &["passport", "license", "document", "identity", "carte", "id"];
    let marker: &[&str] = &["marker", "landmark", "repère", "monument"];
    let business: &[&str] = &[
        "company",
        "business",
        "enterprise",
        "entreprise",
        "shop",
        "store",
    ];
    let place: &[&str] = &["address", "location", "place", "lieu", "city", "ville"];
    let map = words.iter().any(|word| word == "map");
    let game = words.iter().any(|word| word == "game" || word == "gaming");
    if map && game {
        return 2;
    }
    let has_document = words.iter().any(|word| document.contains(&word.as_str()));
    let has_marker = words.iter().any(|word| marker.contains(&word.as_str()));
    let has_business = words.iter().any(|word| business.contains(&word.as_str()));
    let has_place = words.iter().any(|word| place.contains(&word.as_str()));
    let categories = [has_document, has_marker, has_business, has_place]
        .into_iter()
        .filter(|present| *present)
        .count();
    match categories {
        0 | 1 => 0,
        2 if has_marker && has_place && !has_document && !has_business => 0,
        2 => 4,
        _ => 6,
    }
}

fn coordinate_signal(text: &str) -> Option<PrivacyClassification> {
    let numbers = text
        .split(|character: char| {
            !(character.is_ascii_digit() || matches!(character, '.' | '-' | '+'))
        })
        .filter_map(|part| {
            let number = part.parse::<f64>().ok()?;
            number
                .is_finite()
                .then_some((number, decimal_precision(part)))
        })
        .collect::<Vec<_>>();
    let coordinate_hint = text.to_ascii_lowercase();
    numbers.windows(2).find_map(|pair| {
        let decimal_pair =
            pair[0].0.abs() <= 90.0 && pair[1].0.abs() <= 180.0 && pair[0].1 > 0 && pair[1].1 > 0;
        if !decimal_pair {
            return None;
        }
        let explicit_hint = has_numeric_comma_pair(text)
            || coordinate_hint
                .split(|character: char| !character.is_ascii_alphanumeric())
                .any(|word| {
                    matches!(
                        word,
                        "gps"
                            | "lat"
                            | "latitude"
                            | "lon"
                            | "long"
                            | "longitude"
                            | "coord"
                            | "coordinates"
                    )
                });
        if explicit_hint || (pair[0].1 >= 4 && pair[1].1 >= 4 && pair[0].0.abs() >= 20.0) {
            Some(PrivacyClassification::Sensitive)
        } else if pair[0].1 >= 3 && pair[1].1 >= 3 {
            Some(PrivacyClassification::Suspicious)
        } else {
            None
        }
    })
}

#[allow(dead_code)]
fn partial_address_signal(text: &str) -> bool {
    static HOUSE_NUMBER: OnceLock<Regex> = OnceLock::new();
    let house_number =
        HOUSE_NUMBER.get_or_init(|| Regex::new(r"\d{1,6}").expect("valid house number matcher"));
    const STREET_PREFIXES: &[&str] = &[
        "street",
        "rue",
        "road",
        "avenue",
        "boulevard",
        "drive",
        "lane",
        "route",
        "way",
        "place",
        "chemin",
        "allee",
    ];
    for number in house_number.find_iter(text) {
        let before = text[..number.start()].chars().next_back();
        let after = text[number.end()..].chars().next();
        if before.is_some_and(char::is_alphanumeric) || after.is_some_and(char::is_numeric) {
            continue;
        }
        let component = address_component(&text[number.end()..]);
        if component
            .chars()
            .filter(|character| character.is_alphabetic())
            .count()
            < 3
        {
            continue;
        }
        if STREET_PREFIXES
            .iter()
            .any(|prefix| component.starts_with(prefix))
        {
            return true;
        }
        let prefix = &text[..number.start()];
        if (is_leading_address_fragment(prefix) && !is_obvious_non_address_component(&component))
            || has_address_prefix(prefix)
        {
            return true;
        }
    }
    false
}

fn is_leading_address_fragment(prefix: &str) -> bool {
    normalize_rule_words(prefix).is_empty()
}

fn is_obvious_non_address_component(component: &str) -> bool {
    matches!(
        component,
        "minute"
            | "minutes"
            | "seconde"
            | "secondes"
            | "second"
            | "seconds"
            | "heure"
            | "heures"
            | "hour"
            | "hours"
            | "jour"
            | "jours"
            | "day"
            | "days"
            | "semaine"
            | "semaines"
            | "week"
            | "weeks"
            | "mois"
            | "month"
            | "months"
            | "an"
            | "ans"
            | "year"
            | "years"
            | "euro"
            | "euros"
            | "dollar"
            | "dollars"
            | "kilometre"
            | "kilometres"
            | "kilometer"
            | "kilometers"
    )
}

fn address_component(value: &str) -> String {
    let mut compact = String::new();
    for token in value.split_whitespace().take(8) {
        let normalized = normalize_compact(token);
        if normalized.is_empty() {
            continue;
        }
        compact.push_str(&normalized);
        if normalized.chars().count() >= 3 {
            break;
        }
    }
    compact
}

fn has_address_prefix(value: &str) -> bool {
    normalize_rule_words(value)
        .iter()
        .rev()
        .take(4)
        .any(|word| {
            matches!(
                word.as_str(),
                "address"
                    | "adresse"
                    | "domicile"
                    | "home"
                    | "house"
                    | "residence"
                    | "habite"
                    | "jhabite"
                    | "live"
                    | "lives"
                    | "living"
                    | "reside"
                    | "resides"
                    | "chez"
            )
        })
}

fn contains_ip_address(text: &str) -> bool {
    static IPV4: OnceLock<Regex> = OnceLock::new();
    let ipv4 = IPV4.get_or_init(|| {
        Regex::new(
            r"(?:25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])(?:\.(?:25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])){3}",
        )
        .expect("valid IPv4 matcher")
    });
    ipv4.find_iter(text).any(|candidate| {
        let before = text[..candidate.start()].chars().next_back();
        let after = text[candidate.end()..].chars().next();
        if before.is_some_and(|character| character.is_ascii_alphanumeric() || character == '.')
            || after.is_some_and(|character| character.is_ascii_alphanumeric() || character == '.')
        {
            return false;
        }
        !matches!(
            normalize_rule_words(&text[..candidate.start()])
                .last()
                .map(String::as_str),
            Some("version" | "ver" | "v")
        ) && candidate.as_str().parse::<Ipv4Addr>().is_ok()
    }) || text
        .split(|character: char| !character.is_ascii_hexdigit() && character != ':')
        .any(|candidate| {
            candidate.contains(':')
                && candidate.parse::<Ipv6Addr>().is_ok()
                && candidate.parse::<IpAddr>().is_ok()
        })
}

#[allow(dead_code)]
fn location_profile_signal(text: &str, words: &[String]) -> Option<PrivacyClassification> {
    static CITY_FIELD: OnceLock<Regex> = OnceLock::new();
    static COUNTRY_FIELD: OnceLock<Regex> = OnceLock::new();
    let city = CITY_FIELD.get_or_init(|| {
        Regex::new(r#"(?iu)\b(?:city|ville)\s*[:=]\s*[\"']?\p{L}"#)
            .expect("valid city field matcher")
    });
    let country = COUNTRY_FIELD.get_or_init(|| {
        Regex::new(r#"(?iu)\b(?:country|pays)\s*[:=]\s*[\"']?\p{L}"#)
            .expect("valid country field matcher")
    });
    if !city.is_match(text) || !country.is_match(text) {
        return None;
    }
    let personal_location_context = words.iter().any(|word| {
        matches!(
            word.as_str(),
            "address"
                | "adresse"
                | "apartment"
                | "appartement"
                | "domicile"
                | "home"
                | "house"
                | "office"
                | "workplace"
                | "bureau"
                | "residence"
                | "résidence"
        )
    });
    Some(if personal_location_context {
        PrivacyClassification::Sensitive
    } else {
        PrivacyClassification::Suspicious
    })
}

fn has_numeric_comma_pair(text: &str) -> bool {
    let numeric = |value: &str| {
        !value.is_empty()
            && value
                .chars()
                .all(|character| character.is_ascii_digit() || matches!(character, '.' | '-' | '+'))
            && value.parse::<f64>().is_ok()
    };
    for (index, character) in text.char_indices() {
        if character != ',' {
            continue;
        }
        let left = text[..index]
            .trim_end()
            .rsplit(|character: char| {
                !(character.is_ascii_digit() || matches!(character, '.' | '-' | '+'))
            })
            .next()
            .unwrap_or_default();
        let right = text[index + character.len_utf8()..]
            .trim_start()
            .split(|character: char| {
                !(character.is_ascii_digit() || matches!(character, '.' | '-' | '+'))
            })
            .next()
            .unwrap_or_default();
        if numeric(left) && numeric(right) {
            return true;
        }
    }
    false
}

fn decimal_precision(value: &str) -> usize {
    value
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len())
}

fn cap_text(value: &str) -> (&str, bool) {
    value
        .char_indices()
        .nth(PRIVACY_TEXT_LIMIT)
        .map_or((value, false), |(index, _)| (&value[..index], true))
}

fn normalize_words(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(normalize_compact)
        .filter(|word| !word.is_empty())
        .collect()
}

fn normalize_rule_words(value: &str) -> Vec<String> {
    value
        .split_whitespace()
        .map(|segment| {
            segment
                .chars()
                .flat_map(char::to_lowercase)
                .filter_map(|character| {
                    let mapped = match character {
                        'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
                        'ç' => 'c',
                        'è' | 'é' | 'ê' | 'ë' => 'e',
                        'ì' | 'í' | 'î' | 'ï' => 'i',
                        'ñ' => 'n',
                        'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
                        'ù' | 'ú' | 'û' | 'ü' => 'u',
                        'ý' | 'ÿ' => 'y',
                        character if character.is_ascii_alphanumeric() => character,
                        character if character.is_alphanumeric() => character,
                        _ => return None,
                    };
                    Some(mapped)
                })
                .collect::<String>()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

fn normalize_compact(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .filter_map(|character| {
            let mapped = match character {
                'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
                'ç' => 'c',
                'è' | 'é' | 'ê' | 'ë' => 'e',
                'ì' | 'í' | 'î' | 'ï' => 'i',
                'ñ' => 'n',
                'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
                'ù' | 'ú' | 'û' | 'ü' => 'u',
                'ý' | 'ÿ' => 'y',
                'і' | 'ӏ' | '1' => 'i',
                'е' | 'ё' | '3' => 'e',
                'а' | '4' => 'a',
                'о' | '0' => 'o',
                'ѕ' | '5' => 's',
                'т' | '7' => 't',
                'х' => 'x',
                'р' => 'p',
                'с' => 'c',
                character if character.is_ascii_alphanumeric() => character,
                character if character.is_alphanumeric() => character,
                _ => return None,
            };
            Some(mapped)
        })
        .collect()
}

#[derive(Default)]
struct ConceptMatchSignals {
    exact: bool,
    regex: bool,
    similarities: u8,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ConceptMatchKind {
    Exact,
    Similarity,
}

fn forbidden_concept_match(
    text: &str,
    words: &[String],
    concepts: &[ForbiddenConcept],
) -> ConceptMatchSignals {
    let mut signals = ConceptMatchSignals::default();
    for concept in concepts {
        if concept
            .regexes
            .iter()
            .any(|pattern| compile_filter_regex(pattern).is_ok_and(|regex| regex.is_match(text)))
        {
            signals.regex = true;
            return signals;
        }

        let mut variants = vec![concept.canonical.as_str()];
        variants.extend(concept.aliases.iter().map(String::as_str));
        let mut similar = false;
        for variant in variants {
            let normalized = normalize_compact(variant);
            if normalized.chars().count() < 3 {
                continue;
            }
            let phrase = normalize_words(variant);
            let expected = if phrase.len() > 1 {
                phrase.join("")
            } else {
                normalized
            };
            match variant_match(words, &expected) {
                Some(ConceptMatchKind::Exact) => {
                    signals.exact = true;
                    return signals;
                }
                Some(ConceptMatchKind::Similarity) => similar = true,
                None => {}
            }
        }
        if similar {
            signals.similarities = signals.similarities.saturating_add(1);
        }
    }
    signals
}

fn variant_match(words: &[String], expected: &str) -> Option<ConceptMatchKind> {
    let mut similar = false;
    for word in words {
        match concept_word_match(word, expected) {
            Some(ConceptMatchKind::Exact) => return Some(ConceptMatchKind::Exact),
            Some(ConceptMatchKind::Similarity) => similar = true,
            None => {}
        }
    }
    for matcher in [
        composite_split_match(words, expected),
        separated_letter_match(words, expected),
    ] {
        match matcher {
            Some(ConceptMatchKind::Exact) => return Some(ConceptMatchKind::Exact),
            Some(ConceptMatchKind::Similarity) => similar = true,
            None => {}
        }
    }
    similar.then_some(ConceptMatchKind::Similarity)
}

fn concept_word_match(candidate: &str, expected: &str) -> Option<ConceptMatchKind> {
    if candidate == expected {
        return Some(ConceptMatchKind::Exact);
    }
    let reduced = collapse_repeated_runs(candidate);
    if expected.chars().count() >= 6 && reduced != candidate && reduced == expected {
        return Some(ConceptMatchKind::Similarity);
    }
    (cautious_distance(candidate, expected)
        || (reduced != candidate && cautious_distance(&reduced, expected)))
    .then_some(ConceptMatchKind::Similarity)
}

fn composite_split_match(words: &[String], expected: &str) -> Option<ConceptMatchKind> {
    let expected_len = expected.chars().count();
    if expected_len < 3 {
        return None;
    }
    let mut similar = false;
    for start in 0..words.len() {
        let mut candidate = String::new();
        for (count, word) in words
            .iter()
            .skip(start)
            .take(expected_len.saturating_add(1))
            .enumerate()
        {
            if word.is_empty() {
                break;
            }
            candidate.push_str(word);
            let candidate_len = candidate.chars().count();
            if candidate_len > expected_len + 1 {
                break;
            }
            if count >= 1 {
                match concept_word_match(&candidate, expected) {
                    Some(ConceptMatchKind::Exact) => return Some(ConceptMatchKind::Exact),
                    Some(ConceptMatchKind::Similarity) => similar = true,
                    None => {}
                }
            }
        }
    }
    similar.then_some(ConceptMatchKind::Similarity)
}

fn collapse_repeated_runs(value: &str) -> String {
    let mut reduced = String::with_capacity(value.len());
    for character in value.chars() {
        if !reduced.ends_with(character) {
            reduced.push(character);
        }
    }
    reduced
}

fn cautious_distance(left: &str, right: &str) -> bool {
    let length = left.chars().count().max(right.chars().count());
    if length < 6 || left.chars().count().abs_diff(right.chars().count()) > 1 {
        return false;
    }
    if bounded_edit_distance(left, right) != 1 {
        return false;
    }
    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();
    if left_chars.len() == right_chars.len() {
        let differing = left_chars
            .iter()
            .zip(right_chars.iter())
            .filter(|(left, right)| left != right)
            .collect::<Vec<_>>();
        return differing.len() == 1 && confusable_pair(*differing[0].0, *differing[0].1);
    }
    let (longer, shorter) = if left_chars.len() > right_chars.len() {
        (&left_chars, &right_chars)
    } else {
        (&right_chars, &left_chars)
    };
    let insertion = (0..longer.len()).find(|index| {
        longer[..*index]
            .iter()
            .chain(longer[index + 1..].iter())
            .eq(shorter.iter())
    });
    insertion.is_some_and(|index| {
        let inserted = longer[index];
        (index > 0 && longer[index - 1] == inserted)
            || (index + 1 < longer.len() && longer[index + 1] == inserted)
    })
}

fn confusable_pair(left: char, right: char) -> bool {
    matches!(
        (left, right),
        ('l', 'i')
            | ('i', 'l')
            | ('l', '1')
            | ('1', 'l')
            | ('e', '3')
            | ('3', 'e')
            | ('e', 'i')
            | ('i', 'e')
    )
}

fn separated_letter_match(words: &[String], expected: &str) -> Option<ConceptMatchKind> {
    let expected_len = expected.chars().count();
    if expected_len < 3 {
        return None;
    }
    let mut similar = false;
    for window in words.windows(expected_len) {
        if !window.iter().all(|word| word.chars().count() == 1) {
            continue;
        }
        match concept_word_match(&window.join(""), expected) {
            Some(ConceptMatchKind::Exact) => return Some(ConceptMatchKind::Exact),
            Some(ConceptMatchKind::Similarity) => similar = true,
            None => {}
        }
    }
    similar.then_some(ConceptMatchKind::Similarity)
}

fn bounded_edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut row = (0..=right.len()).collect::<Vec<_>>();
    for (index, left_char) in left.chars().enumerate() {
        let mut next = vec![index + 1; right.len() + 1];
        for (right_index, right_char) in right.iter().enumerate() {
            next[right_index + 1] = (row[right_index + 1] + 1)
                .min(next[right_index] + 1)
                .min(row[right_index] + usize::from(left_char != *right_char));
        }
        row = next;
    }
    row[right.len()]
}

pub fn action_for(report: &PrivacyReport, config: &AppConfig) -> PrivacyAction {
    if report.classification == PrivacyClassification::Critical
        || report.classification.rank() >= config.privacy_block_threshold.rank()
    {
        return PrivacyAction::Block;
    }
    match report.classification {
        PrivacyClassification::High => PrivacyAction::Review,
        PrivacyClassification::Medium if config.privacy_review_intermediate => {
            PrivacyAction::Review
        }
        PrivacyClassification::Safe
        | PrivacyClassification::Low
        | PrivacyClassification::Medium => PrivacyAction::Allow,
        PrivacyClassification::Critical => PrivacyAction::Block,
    }
}

pub fn log_decision(report: &PrivacyReport, action: PrivacyAction) {
    if matches!(action, PrivacyAction::Allow)
        || report.classification == PrivacyClassification::Safe
    {
        return;
    }
    let detected = if report.categories.is_empty() {
        "UNCLASSIFIED".to_owned()
    } else {
        report
            .categories
            .iter()
            .map(|category| category.log_code())
            .collect::<Vec<_>>()
            .join("+")
    };
    let action = match action {
        PrivacyAction::Allow => "ALLOWED",
        PrivacyAction::Review => "REVIEW",
        PrivacyAction::Block => "BLOCKED",
    };
    eprintln!(
        "Privacy Risk: {} Detected: {detected} Action: {action}",
        report.classification.log_code()
    );
}

/// Text filters remain active even when the optional image metadata/OCR scan
/// is disabled. The latter is deliberately kept separate so filter-only
/// configurations do not make ordinary images incomplete by default.
pub fn privacy_rules_enabled(config: &AppConfig) -> bool {
    config.privacy_scan_enabled || !config.privacy_concepts.is_empty()
}

pub fn has_exempt_role(config: &AppConfig, message_role_ids: &[String]) -> bool {
    !message_role_ids.is_empty()
        && config
            .privacy_filter_exempt_role_ids
            .iter()
            .any(|configured| message_role_ids.iter().any(|role| role == configured))
}

pub fn scoped_config_for_roles(config: &AppConfig, message_role_ids: &[String]) -> AppConfig {
    if !has_exempt_role(config, message_role_ids) {
        return config.clone();
    }
    let mut scoped = config.clone();
    scoped.privacy_concepts.clear();
    scoped
}

pub fn config_signature(config: &AppConfig) -> u64 {
    let mut hasher = DefaultHasher::new();
    config.privacy_scan_enabled.hash(&mut hasher);
    (config.privacy_suspicious_policy as u8).hash(&mut hasher);
    config.privacy_suspicious_threshold.hash(&mut hasher);
    config.privacy_sensitive_threshold.hash(&mut hasher);
    config.privacy_similarity_boost.hash(&mut hasher);
    config.privacy_protection_level.hash(&mut hasher);
    config.privacy_block_threshold.hash(&mut hasher);
    config.privacy_review_intermediate.hash(&mut hasher);
    for category in &config.privacy_enabled_categories {
        category.hash(&mut hasher);
    }
    for value in &config.privacy_allowlist {
        value.hash(&mut hasher);
    }
    for value in &config.privacy_custom_patterns {
        value.hash(&mut hasher);
    }
    for role_id in &config.privacy_filter_exempt_role_ids {
        role_id.hash(&mut hasher);
    }
    for concept in &config.privacy_concepts {
        concept.canonical.hash(&mut hasher);
        for alias in &concept.aliases {
            alias.hash(&mut hasher);
        }
        for pattern in &concept.regexes {
            pattern.hash(&mut hasher);
        }
    }
    hasher.finish()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivacyAction {
    Allow,
    Review,
    Block,
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_PIXEL_PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60,
        0x60, 0x60, 0x00, 0x00, 0x00, 0x04, 0x00, 0x01, 0x27, 0x34, 0x13, 0xa6, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    fn jpeg_with_gps_metadata() -> Vec<u8> {
        let mut tiff = vec![0x49, 0x49, 0x2a, 0x00, 0x08, 0x00, 0x00, 0x00];
        tiff.extend_from_slice(&[0x01, 0x00, 0x25, 0x88, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00]);
        tiff.extend_from_slice(&[0x1a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
        tiff.extend_from_slice(&[0x02, 0x00]);
        tiff.extend_from_slice(&[
            0x01, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x00, b'N', 0x00, 0x00, 0x00,
        ]);
        tiff.extend_from_slice(&[
            0x02, 0x00, 0x05, 0x00, 0x03, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00, 0x00,
        ]);
        tiff.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        tiff.extend_from_slice(&[
            1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0,
        ]);
        let payload_len = 6 + tiff.len();
        let segment_len = payload_len + 2;
        let mut jpeg = vec![
            0xff,
            0xd8,
            0xff,
            0xe1,
            (segment_len as u16 >> 8) as u8,
            segment_len as u8,
        ];
        jpeg.extend_from_slice(b"Exif\0\0");
        jpeg.extend_from_slice(&tiff);
        jpeg.extend_from_slice(&[0xff, 0xd9]);
        jpeg
    }

    fn jpeg_with_model_metadata() -> Vec<u8> {
        let mut tiff = vec![0x49, 0x49, 0x2a, 0x00, 0x08, 0x00, 0x00, 0x00];
        tiff.extend_from_slice(&[0x01, 0x00]);
        tiff.extend_from_slice(&[
            0x10, 0x01, 0x02, 0x00, 0x06, 0x00, 0x00, 0x00, 0x1a, 0x00, 0x00, 0x00,
        ]);
        tiff.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        tiff.extend_from_slice(b"Phone\0");
        let payload_len = 6 + tiff.len();
        let segment_len = payload_len + 2;
        let mut jpeg = vec![
            0xff,
            0xd8,
            0xff,
            0xe1,
            (segment_len as u16 >> 8) as u8,
            segment_len as u8,
        ];
        jpeg.extend_from_slice(b"Exif\0\0");
        jpeg.extend_from_slice(&tiff);
        jpeg.extend_from_slice(&[0xff, 0xd9]);
        jpeg
    }

    fn config() -> AppConfig {
        AppConfig {
            privacy_scan_enabled: true,
            privacy_concepts: vec![ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: vec!["austrian painter".into()],
                regexes: Vec::new(),
            }],
            ..AppConfig::default()
        }
    }

    #[test]
    fn safe_text_does_not_overclassify() {
        let config = config();
        for text in [
            "a landscape",
            "generic building",
            "public monument",
            "meme 123 456",
        ] {
            assert_eq!(
                classify_text(Some(text), &config).classification,
                PrivacyClassification::Safe
            );
        }
    }

    #[test]
    fn address_and_coordinates_are_sensitive() {
        let config = config();
        assert_eq!(
            classify_text(Some("12 Main Street 75001 Paris"), &config).classification,
            PrivacyClassification::Sensitive
        );
        assert_eq!(
            classify_text(Some("48.8566, 2.3522"), &config).classification,
            PrivacyClassification::Sensitive
        );
    }

    #[test]
    fn probable_addresses_are_sensitive_across_common_formats() {
        let config = config();
        for text in [
            "1 rue canot massy",
            "1, rue Canot - Massy",
            "1 r.u.e Canot Massy",
            "adresse : 1 Canot Massy",
            "6\nrue\ncanot\nmassy\n91300",
        ] {
            assert_eq!(
                classify_text(Some(text), &config).classification,
                PrivacyClassification::High,
                "{text}"
            );
        }
    }

    #[test]
    fn partial_addresses_stay_low_without_overclassifying_ordinary_numbers() {
        let config = config();
        for text in ["5 rue", "5 avenue", "5 r.u.e"] {
            let report = classify_text(Some(text), &config);
            assert_eq!(report.classification, PrivacyClassification::Low, "{text}");
            assert!(report.reasons.contains(&"partial_address"), "{text}");
        }
        for text in [
            "5 martin",
            "5 m-a-r-t-i-n",
            "5 minutes",
            "5 euros",
            "5 kilometres",
            "version 1.2.3.4",
            "meme 123 456",
        ] {
            assert_eq!(
                classify_text(Some(text), &config).classification,
                PrivacyClassification::Safe,
                "{text}"
            );
        }
    }

    #[test]
    fn doxxing_text_formats_classify_contextually() {
        let config = config();
        for text in ["IP: 203.0.113.42", "[2001:db8::1]"] {
            assert_eq!(
                classify_text(Some(text), &config).classification,
                PrivacyClassification::Medium,
                "{text}"
            );
        }
        assert_eq!(
            classify_text(Some("12 Main Street 75001 Paris"), &config).classification,
            PrivacyClassification::High
        );
        for text in [
            "version 1.2.3",
            "Paris, France",
            "public monument in a city",
        ] {
            assert_eq!(
                classify_text(Some(text), &config).classification,
                PrivacyClassification::Safe,
                "{text}"
            );
        }
    }

    #[test]
    fn game_map_is_never_sensitive() {
        let config = config();
        assert_ne!(
            classify_text(Some("game map"), &config).classification,
            PrivacyClassification::Sensitive
        );
    }

    #[test]
    fn concepts_match_explicit_variants_only() {
        let config = config();
        for text in [
            "hitler",
            "h1tler",
            "hitier",
            "h1tl3r",
            "h3tler",
            "hiiitler",
            "hi tler",
            "hit ler",
            "austrian painter",
            "austrian-painter",
            "aus trian painter",
            "a u s t r i a n painter",
            "h i t l e r",
        ] {
            let report = classify_text(Some(text), &config);
            assert!(
                report.classification.rank() >= PrivacyClassification::High.rank(),
                "{text}"
            );
            assert_eq!(action_for(&report, &config), PrivacyAction::Block, "{text}");
        }
        assert_eq!(
            classify_text(Some("hit"), &config).classification,
            PrivacyClassification::Safe
        );
        assert_eq!(
            classify_text(Some("unrelated"), &config).classification,
            PrivacyClassification::Safe
        );
        for text in ["hitter", "hither"] {
            assert_eq!(
                classify_text(Some(text), &config).classification,
                PrivacyClassification::Safe,
                "{text}"
            );
        }
        let mut long_text = "a".repeat(200);
        long_text.push_str(" h1tler");
        assert_eq!(
            classify_text(Some(&long_text), &config).classification,
            PrivacyClassification::Critical
        );
        let truncated = "a".repeat(PRIVACY_TEXT_LIMIT + 1);
        let truncated_report = classify_text(Some(&truncated), &config);
        assert_eq!(truncated_report.classification, PrivacyClassification::Low);
        assert!(truncated_report.reasons.contains(&"scan_incomplete"));
        let concept_before_cap = format!("h1tler {}", "a".repeat(PRIVACY_TEXT_LIMIT + 1));
        let concept_report = classify_text(Some(&concept_before_cap), &config);
        assert_eq!(
            concept_report.classification,
            PrivacyClassification::Critical
        );
        assert!(concept_report.reasons.contains(&"scan_incomplete"));
        let punctuation_report = classify_text(
            Some(&format!("{}hitler", ".".repeat(PRIVACY_TEXT_LIMIT + 1))),
            &config,
        );
        assert_eq!(
            punctuation_report.classification,
            PrivacyClassification::Low
        );
        assert!(punctuation_report.reasons.contains(&"scan_incomplete"));
        assert!(
            ForbiddenConcept {
                canonical: "1234".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }
            .validate()
            .is_err()
        );
        assert!(
            ForbiddenConcept {
                canonical: "okay".into(),
                aliases: vec!["5678".into()],
                regexes: Vec::new(),
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn regex_filters_block_directly_and_similarity_boosts_score() {
        let mut config = config();
        config.privacy_scan_enabled = false;
        config.privacy_concepts[0].regexes = vec![r"\bsecret[-_ ]phrase\b".into()];
        let regex_report = classify_text(Some("SECRET_phrase"), &config);
        assert_eq!(regex_report.classification, PrivacyClassification::Critical);
        assert!(regex_report.reasons.contains(&"forbidden_regex"));
        config.privacy_concepts[0].regexes = vec![r"^---$".into()];
        assert_eq!(
            classify_text(Some("---"), &config).classification,
            PrivacyClassification::Critical
        );

        config.privacy_concepts[0].regexes.clear();
        config.privacy_similarity_boost = 2;
        let weak_similarity = classify_text(Some("hitier"), &config);
        assert_eq!(
            weak_similarity.classification,
            PrivacyClassification::Suspicious
        );
        assert!(weak_similarity.reasons.contains(&"forbidden_similarity"));

        config.privacy_similarity_boost = 4;
        let strong_similarity = classify_text(Some("hitier"), &config);
        assert_eq!(
            strong_similarity.classification,
            PrivacyClassification::Sensitive
        );
        assert!(strong_similarity.reasons.contains(&"similarity_score"));

        for regexes in [vec!["(".into()], vec![".*".into()]] {
            assert!(
                ForbiddenConcept {
                    canonical: "filter".into(),
                    aliases: Vec::new(),
                    regexes,
                }
                .validate()
                .is_err()
            );
        }
    }

    #[test]
    fn three_character_filter_words_match_exact_separators_only() {
        let filter_config = AppConfig {
            privacy_scan_enabled: false,
            privacy_concepts: vec![ForbiddenConcept {
                canonical: "fdp".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }],
            ..AppConfig::default()
        };
        for text in ["fdp", "f.d.p", "f-d-p", "f d p"] {
            assert_eq!(
                classify_text(Some(text), &filter_config).classification,
                PrivacyClassification::Critical,
                "{text}"
            );
        }
        for text in ["fd", "ffdp", "fdpp", "unrelated"] {
            assert_eq!(
                classify_text(Some(text), &filter_config).classification,
                PrivacyClassification::Safe,
                "{text}"
            );
        }
        assert!(
            ForbiddenConcept {
                canonical: "fdp".into(),
                aliases: vec!["f.d.p".into()],
                regexes: Vec::new(),
            }
            .validate()
            .is_ok()
        );
        assert!(
            ForbiddenConcept {
                canonical: "fd".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }
            .validate()
            .is_err()
        );
        assert!(
            ForbiddenConcept {
                canonical: "filter".into(),
                aliases: vec!["fd".into()],
                regexes: Vec::new(),
            }
            .validate()
            .is_err()
        );
        assert_eq!(
            classify_text(Some("h1tl3r"), &config()).classification,
            PrivacyClassification::Critical
        );
    }

    #[test]
    fn disabled_scan_bypasses_all_rules() {
        let mut config = config();
        config.privacy_scan_enabled = false;
        config.privacy_concepts.clear();
        assert_eq!(
            classify_text(Some("12 Main Street 75001 Paris hitler"), &config).classification,
            PrivacyClassification::Safe
        );
    }

    #[test]
    fn filter_words_run_without_local_image_scan() {
        let mut config = config();
        config.privacy_scan_enabled = false;
        assert!(privacy_rules_enabled(&config));
        assert_eq!(
            classify_text(Some("hitler"), &config).classification,
            PrivacyClassification::Critical
        );

        config.privacy_concepts[0].canonical = "private".into();
        config.privacy_concepts[0].aliases.clear();
        config.privacy_concepts[0].regexes = vec![r"\bsecret\b".into()];
        let regex_report = classify_text(Some("a SECRET message"), &config);
        assert_eq!(regex_report.classification, PrivacyClassification::Critical);
        assert!(regex_report.reasons.contains(&"forbidden_regex"));
    }

    #[test]
    fn exempt_roles_clear_only_filter_concepts() {
        let mut config = config();
        config.privacy_scan_enabled = true;
        config.privacy_filter_exempt_role_ids = vec!["123456789012345678".into()];
        let roles = vec!["123456789012345678".into()];
        assert!(has_exempt_role(&config, &roles));
        let scoped = scoped_config_for_roles(&config, &roles);
        assert!(scoped.privacy_concepts.is_empty());
        assert!(scoped.privacy_scan_enabled);
        assert_eq!(
            classify_text(Some("hitler"), &scoped).classification,
            PrivacyClassification::Safe
        );
        assert_eq!(
            classify_text(Some("gps 48.8566, 2.3522"), &scoped).classification,
            PrivacyClassification::Sensitive
        );
        assert_eq!(
            classify_text(Some("203.0.113.42"), &scoped).classification,
            PrivacyClassification::Medium
        );
        assert!(!has_exempt_role(&config, &Vec::new()));
        assert!(!has_exempt_role(&config, &["223456789012345678".into()]));
        let signature = config_signature(&config);
        let mut changed = config.clone();
        changed.privacy_filter_exempt_role_ids = vec!["223456789012345678".into()];
        assert_ne!(signature, config_signature(&changed));
    }

    #[test]
    fn filter_signals_can_be_removed_without_dropping_gps() {
        let mut report = PrivacyReport::sensitive("forbidden_concept");
        report.merge(PrivacyReport::sensitive("gps"));
        let scoped = report.without_filter_signals();
        assert_eq!(scoped.classification, PrivacyClassification::Sensitive);
        assert_eq!(scoped.reasons, vec!["gps"]);

        let report = PrivacyReport::sensitive("forbidden_regex");
        let scoped = report.without_filter_signals();
        assert_eq!(scoped.classification, PrivacyClassification::Safe);
        assert!(scoped.reasons.is_empty());

        let custom = PrivacyReport::sensitive("custom_pattern");
        let scoped = custom.clone().without_filter_signals();
        assert_eq!(scoped, custom);
    }

    #[test]
    fn common_visual_text_stays_safe_or_weak() {
        let config = config();
        for text in [
            "Street Fighter 6 meme",
            "public monument city",
            "landscape city landmark",
            "meme 12.5 3.2",
        ] {
            assert_eq!(
                classify_text(Some(text), &config).classification,
                PrivacyClassification::Safe,
                "{text}"
            );
        }
        assert_ne!(
            classify_text(Some("meme, 12.5 3.2"), &config).classification,
            PrivacyClassification::Sensitive
        );
        assert_ne!(
            classify_text(Some("flat meme 12.5 3.2"), &config).classification,
            PrivacyClassification::Sensitive
        );
        assert_eq!(
            classify_text(Some("gps 48.8566, 2.3522"), &config).classification,
            PrivacyClassification::Sensitive
        );
        assert_eq!(
            classify_text(Some("passport landmark company city"), &config).classification,
            PrivacyClassification::Safe
        );
    }

    #[test]
    fn contact_signals_are_scored_without_treating_ordinary_numbers_as_phone_numbers() {
        let config = config();
        for email in ["person@example.com", "person [at] example [dot] com"] {
            let report = classify_text(Some(email), &config);
            assert_eq!(report.classification, PrivacyClassification::Low, "{email}");
            assert!(report.categories.contains(&PrivacyCategory::Email));
        }
        for phone in ["06 12 34 56 78", "+33 (0)6 12 34 56 78"] {
            let report = classify_text(Some(phone), &config);
            assert_eq!(
                report.classification,
                PrivacyClassification::Medium,
                "{phone}"
            );
            assert!(report.categories.contains(&PrivacyCategory::Phone));
        }
        for ordinary in [
            "order 123456",
            "version 1.2.3.4",
            "Discord 123456789012345678",
        ] {
            assert_eq!(
                classify_text(Some(ordinary), &config).classification,
                PrivacyClassification::Safe,
                "{ordinary}"
            );
        }
    }

    #[test]
    fn financial_identifiers_use_checksum_validation() {
        let config = config();
        let iban = classify_text(Some("GB82 WEST 1234 5698 7654 32"), &config);
        assert_eq!(iban.classification, PrivacyClassification::High);
        assert!(iban.reasons.contains(&"iban"));

        let card = classify_text(Some("4111 1111 1111 1111"), &config);
        assert_eq!(card.classification, PrivacyClassification::Critical);
        assert!(card.reasons.contains(&"payment_card"));

        for invalid in ["GB82 WEST 1234 5698 7654 31", "4111 1111 1111 1112"] {
            assert_eq!(
                classify_text(Some(invalid), &config).classification,
                PrivacyClassification::Safe,
                "{invalid}"
            );
        }
    }

    #[test]
    fn context_combinations_raise_risk_and_plate_detection_stays_bounded() {
        let config = config();
        let combined = classify_text(
            Some("name: Jean Dupont, 12 rue Victor Hugo 75001 Paris, 06 12 34 56 78"),
            &config,
        );
        assert_eq!(combined.classification, PrivacyClassification::Critical);
        assert_eq!(action_for(&combined, &config), PrivacyAction::Block);

        let plate = classify_text(Some("plaque AB-123-CD"), &config);
        assert_eq!(plate.classification, PrivacyClassification::Medium);
        assert!(plate.categories.contains(&PrivacyCategory::LicensePlate));
        assert_eq!(
            classify_text(Some("build AB123CD"), &config).classification,
            PrivacyClassification::Safe
        );
    }

    #[test]
    fn sensitive_urls_custom_patterns_allowlist_and_category_toggles_work() {
        let mut config = config();
        let url = classify_text(Some("https://example.com/reset?token=abc123"), &config);
        assert_eq!(url.classification, PrivacyClassification::Medium);
        assert!(url.categories.contains(&PrivacyCategory::SensitiveUrl));

        config.privacy_custom_patterns = vec!["oldnickname".into(), "private street".into()];
        for text in [
            "old\u{200b}nickname",
            "private-street",
            "oldnicknam\u{0435}",
        ] {
            let report = classify_text(Some(text), &config);
            assert_eq!(report.classification, PrivacyClassification::High, "{text}");
            assert!(report.categories.contains(&PrivacyCategory::CustomPattern));
        }

        config.privacy_allowlist = vec!["public@example.com".into()];
        assert_eq!(
            classify_text(Some("public@example.com"), &config).classification,
            PrivacyClassification::Safe
        );
        config.privacy_enabled_categories = vec![PrivacyCategory::Phone];
        assert_eq!(
            classify_text(Some("private@example.com"), &config).classification,
            PrivacyClassification::Safe
        );
    }

    #[test]
    fn protection_profiles_and_actions_follow_the_five_level_policy() {
        assert_eq!(
            risk_for_score(25, ProtectionLevel::Balanced),
            PrivacyClassification::Low
        );
        assert_eq!(
            risk_for_score(25, ProtectionLevel::Strict),
            PrivacyClassification::Medium
        );
        assert_eq!(
            risk_for_score(25, ProtectionLevel::Paranoid),
            PrivacyClassification::Medium
        );
        let mut config = config();
        let medium = PrivacyReport::suspicious("phone");
        assert_eq!(action_for(&medium, &config), PrivacyAction::Review);
        config.privacy_review_intermediate = false;
        assert_eq!(action_for(&medium, &config), PrivacyAction::Allow);
        config.privacy_block_threshold = PrivacyClassification::Critical;
        assert_eq!(
            action_for(&PrivacyReport::sensitive("gps"), &config),
            PrivacyAction::Review
        );
        assert_eq!(
            action_for(
                &PrivacyReport {
                    classification: PrivacyClassification::Critical,
                    score: 100,
                    categories: vec![PrivacyCategory::Financial],
                    reasons: vec!["payment_card"],
                    config_signature: None,
                },
                &config,
            ),
            PrivacyAction::Block
        );

        let mut combined = PrivacyReport::suspicious("phone");
        combined.merge(PrivacyReport::suspicious("postal_address"));
        assert_eq!(combined.classification, PrivacyClassification::Medium);
        combined.apply_score_policy(&config);
        assert_eq!(combined.classification, PrivacyClassification::High);
    }

    #[test]
    fn image_metadata_mime_and_resource_limits_are_classified_locally() {
        let config = config();
        let metadata = analyze_image_bytes(&jpeg_with_model_metadata(), None, &config);
        assert!(
            metadata
                .categories
                .contains(&PrivacyCategory::ImageMetadata)
        );
        assert!(metadata.reasons.contains(&"exif_metadata"));

        let mismatch = analyze_image_bytes(b"not an image", None, &config);
        assert_eq!(mismatch.classification, PrivacyClassification::High);
        assert!(mismatch.reasons.contains(&"mime_mismatch"));

        let too_large = image_limit_report(None, &config);
        assert_eq!(too_large.classification, PrivacyClassification::High);
        assert_eq!(action_for(&too_large, &config), PrivacyAction::Block);
    }

    #[test]
    fn ordinary_image_fixture_is_safe() {
        let mut config = config();
        config.privacy_scan_enabled = false;
        let report = analyze_image_bytes(ONE_PIXEL_PNG, None, &config);
        assert_eq!(report.classification, PrivacyClassification::Safe);
    }

    #[test]
    fn gps_fixture_is_sensitive_before_publication() {
        let report = analyze_image_bytes(&jpeg_with_gps_metadata(), None, &config());
        assert_eq!(report.classification, PrivacyClassification::Sensitive);
        assert!(report.reasons.contains(&"gps"));
    }

    #[test]
    fn malformed_exif_is_reviewable_incomplete() {
        let malformed = b"\xff\xd8\xff\xe1\0\x08Exif\0\0\0";
        let report = analyze_image_bytes(malformed, None, &config());
        assert_eq!(report.classification, PrivacyClassification::Low);
        assert!(report.reasons.contains(&"scan_incomplete"));
    }
}
