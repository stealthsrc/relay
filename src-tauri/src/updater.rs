use std::{
    env,
    fmt::Write as _,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::{Context, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use futures_util::StreamExt;
use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::AppHandle;

#[cfg(target_os = "windows")]
use std::os::windows::fs::OpenOptionsExt;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const LATEST_RELEASE_API: &str = "https://api.github.com/repos/stealthsrc/relay/releases/latest";
const MAX_RELEASE_METADATA_BYTES: usize = 1024 * 1024;
const MAX_INSTALLER_BYTES: u64 = 100 * 1024 * 1024;
const MAX_SIGNATURE_BYTES: usize = 16 * 1024;
const UPDATER_PUBLIC_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEFCM0I0NDU0MDNEMTY2NDEKUldSQlp0RURWRVE3cTMrZ2t0MjlnVWUvMTA1bWpKT3Q0L0F5RDNRcTd2alMrS0lQM1Zyb1VrRmIK";

struct VerifiedInstaller {
    path: PathBuf,
    _lock: File,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    current_version: &'static str,
    latest_version: String,
    update_available: bool,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}

#[tauri::command]
pub fn get_app_version() -> &'static str {
    CURRENT_VERSION
}

#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateStatus, String> {
    let release = fetch_latest_release().await.map_err(display_error)?;
    let latest_version = release_version(&release.tag_name)
        .map_err(display_error)?
        .to_owned();
    let update_available =
        is_newer_version(&latest_version, CURRENT_VERSION).map_err(display_error)?;
    Ok(UpdateStatus {
        current_version: CURRENT_VERSION,
        latest_version,
        update_available,
    })
}

#[tauri::command]
pub async fn download_and_install_update(app: AppHandle) -> Result<(), String> {
    let release = fetch_latest_release().await.map_err(display_error)?;
    let latest_version = release_version(&release.tag_name).map_err(display_error)?;
    if !is_newer_version(latest_version, CURRENT_VERSION).map_err(display_error)? {
        return Err("Relay is already up to date.".into());
    }

    let signature = download_signature(&release).await.map_err(display_error)?;
    let installer = download_installer(&release, &signature)
        .await
        .map_err(display_error)?;
    if let Err(error) = Command::new(&installer.path).spawn() {
        let path = installer.path.clone();
        drop(installer);
        let _ = fs::remove_file(path);
        return Err(format!("Unable to launch the Relay installer: {error}"));
    }
    app.exit(0);
    Ok(())
}

async fn fetch_latest_release() -> anyhow::Result<GitHubRelease> {
    let client = reqwest::Client::builder()
        .user_agent(concat!("Relay/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .build()
        .context("unable to create the update client")?;
    let response = client
        .get(LATEST_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .context("unable to reach GitHub")?
        .error_for_status()
        .context("GitHub rejected the update request")?;
    let body = read_limited_response(response, MAX_RELEASE_METADATA_BYTES)
        .await
        .context("unable to read the GitHub response")?;
    serde_json::from_slice(&body).context("GitHub returned an invalid release response")
}

async fn read_limited_response(
    response: reqwest::Response,
    max_bytes: usize,
) -> anyhow::Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|size| size > max_bytes as u64)
    {
        bail!("the GitHub response is too large");
    }
    let capacity = response.content_length().unwrap_or(0).min(max_bytes as u64) as usize;
    let mut body = Vec::with_capacity(capacity);
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("the GitHub response was interrupted")?;
        let next_size = body
            .len()
            .checked_add(chunk.len())
            .context("the GitHub response size overflowed")?;
        if next_size > max_bytes {
            bail!("the GitHub response is too large");
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn download_signature(release: &GitHubRelease) -> anyhow::Result<String> {
    let version = release_version(&release.tag_name)?;
    let asset = signature_asset(release, version)?;
    if asset.size == 0 || asset.size > MAX_SIGNATURE_BYTES as u64 {
        bail!("the updater signature size is outside the allowed range");
    }
    let expected_name = format!("Relay_{version}_x64-setup.exe.sig");
    let download_url = validated_asset_url(release, asset, &expected_name)?;
    let client = reqwest::Client::builder()
        .user_agent(concat!("Relay/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .build()
        .context("unable to create the signature client")?;
    let response = client
        .get(download_url)
        .send()
        .await
        .context("unable to download the Relay updater signature")?
        .error_for_status()
        .context("GitHub rejected the updater signature download")?;
    let body = read_limited_response(response, MAX_SIGNATURE_BYTES)
        .await
        .context("unable to read the updater signature")?;
    if body.len() as u64 != asset.size {
        bail!("the updater signature download is incomplete");
    }
    let signature = String::from_utf8(body).context("the updater signature is not UTF-8")?;
    if signature.trim().is_empty() {
        bail!("the updater signature is empty");
    }
    Ok(signature.trim().to_owned())
}

async fn download_installer(
    release: &GitHubRelease,
    signature: &str,
) -> anyhow::Result<VerifiedInstaller> {
    let version = release_version(&release.tag_name)?;
    let asset = installer_asset(release, version)?;
    let expected_digest = expected_digest(asset)?;
    let download_url = validated_download_url(release, asset, version)?;
    if asset.size == 0 || asset.size > MAX_INSTALLER_BYTES {
        bail!("the installer size is outside the allowed range");
    }

    let client = reqwest::Client::builder()
        .user_agent(concat!("Relay/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(120))
        .build()
        .context("unable to create the download client")?;
    let response = client
        .get(download_url)
        .send()
        .await
        .context("unable to download the Relay installer")?
        .error_for_status()
        .context("GitHub rejected the installer download")?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_INSTALLER_BYTES)
    {
        bail!("the installer download is too large");
    }

    let nonce = rand::random::<u64>();
    let path = env::temp_dir().join(format!("Relay_{version}_{nonce:016x}_update.exe"));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(target_os = "windows")]
    options.share_mode(0x0000_0001); // FILE_SHARE_READ keeps the verified file immutable.
    let mut file = options
        .open(&path)
        .context("unable to create the temporary installer")?;
    let download_result: anyhow::Result<()> = async {
        let mut received = 0_u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("the installer download was interrupted")?;
            received = received
                .checked_add(chunk.len() as u64)
                .context("the installer size overflowed")?;
            if received > asset.size || received > MAX_INSTALLER_BYTES {
                bail!("the installer download exceeded its declared size");
            }
            file.write_all(&chunk)
                .context("unable to write the temporary installer")?;
        }
        file.sync_all()
            .context("unable to finish the temporary installer")?;
        if received != asset.size {
            bail!("the installer download is incomplete");
        }
        Ok(())
    }
    .await;
    if let Err(error) = download_result {
        drop(file);
        let _ = fs::remove_file(&path);
        return Err(error);
    }
    drop(file);

    let mut lock_options = OpenOptions::new();
    lock_options.read(true);
    #[cfg(target_os = "windows")]
    lock_options.share_mode(0x0000_0001); // Allow reads while denying replacement or deletion.
    let lock = match lock_options.open(&path) {
        Ok(lock) => lock,
        Err(error) => {
            let _ = fs::remove_file(&path);
            return Err(error).context("unable to lock the downloaded installer");
        }
    };

    let actual_digest = match file_sha256(&path) {
        Ok(digest) => digest,
        Err(error) => {
            drop(lock);
            let _ = fs::remove_file(&path);
            return Err(error);
        }
    };
    if !actual_digest.eq_ignore_ascii_case(expected_digest) {
        drop(lock);
        let _ = fs::remove_file(&path);
        bail!("the installer SHA-256 does not match the GitHub release");
    }
    if let Err(error) = verify_file_signature(&path, signature, &asset.name) {
        drop(lock);
        let _ = fs::remove_file(&path);
        return Err(error);
    }
    Ok(VerifiedInstaller { path, _lock: lock })
}

fn installer_asset<'a>(
    release: &'a GitHubRelease,
    version: &str,
) -> anyhow::Result<&'a GitHubAsset> {
    let expected_name = format!("Relay_{version}_x64-setup.exe");
    release
        .assets
        .iter()
        .find(|asset| asset.name == expected_name)
        .context("the latest release has no Windows x64 installer")
}

fn signature_asset<'a>(
    release: &'a GitHubRelease,
    version: &str,
) -> anyhow::Result<&'a GitHubAsset> {
    let expected_name = format!("Relay_{version}_x64-setup.exe.sig");
    release
        .assets
        .iter()
        .find(|asset| asset.name == expected_name)
        .context("the latest release has no signed Windows x64 installer")
}

fn validated_download_url(
    release: &GitHubRelease,
    asset: &GitHubAsset,
    version: &str,
) -> anyhow::Result<reqwest::Url> {
    let expected_name = format!("Relay_{version}_x64-setup.exe");
    validated_asset_url(release, asset, &expected_name)
}

fn validated_asset_url(
    release: &GitHubRelease,
    asset: &GitHubAsset,
    expected_name: &str,
) -> anyhow::Result<reqwest::Url> {
    let expected_path = format!(
        "/stealthsrc/relay/releases/download/{}/{}",
        release.tag_name, expected_name
    );
    let url = reqwest::Url::parse(&asset.browser_download_url)
        .context("the installer has an invalid download URL")?;
    if url.scheme() != "https"
        || url.host_str() != Some("github.com")
        || url.path() != expected_path
        || url.query().is_some()
    {
        bail!("the installer download URL is not an official Relay release asset");
    }
    Ok(url)
}

fn decode_tauri_minisign(value: &str, label: &str) -> anyhow::Result<String> {
    let decoded = BASE64_STANDARD
        .decode(value.trim())
        .with_context(|| format!("the {label} is not valid base64"))?;
    String::from_utf8(decoded).with_context(|| format!("the {label} is not valid UTF-8"))
}

fn verify_file_signature(
    path: &Path,
    encoded_signature: &str,
    expected_name: &str,
) -> anyhow::Result<()> {
    verify_file_signature_with_key(path, encoded_signature, UPDATER_PUBLIC_KEY, expected_name)
}

fn verify_file_signature_with_key(
    path: &Path,
    encoded_signature: &str,
    encoded_public_key: &str,
    expected_name: &str,
) -> anyhow::Result<()> {
    let public_key = decode_tauri_minisign(encoded_public_key, "updater public key")?;
    let public_key = PublicKey::decode(&public_key).context("the updater public key is invalid")?;
    let signature = decode_tauri_minisign(encoded_signature, "updater signature")?;
    let signature = Signature::decode(&signature).context("the updater signature is invalid")?;
    let signed_name = signature
        .trusted_comment()
        .split('\t')
        .find_map(|field| field.strip_prefix("file:"))
        .and_then(|value| value.rsplit(['/', '\\']).next())
        .context("the updater signature does not bind an installer name")?;
    if signed_name != expected_name {
        bail!("the updater signature was created for a different installer");
    }
    let mut verifier = public_key
        .verify_stream(&signature)
        .context("the updater signature does not support streaming verification")?;
    let mut file = File::open(path).context("unable to read the downloaded installer")?;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .context("unable to verify the downloaded installer")?;
        if read == 0 {
            break;
        }
        verifier.update(&buffer[..read]);
    }
    verifier
        .finalize()
        .context("the installer signature is not trusted")
}

fn expected_digest(asset: &GitHubAsset) -> anyhow::Result<&str> {
    let digest = asset
        .digest
        .as_deref()
        .and_then(|value| value.strip_prefix("sha256:"))
        .context("the installer has no GitHub SHA-256 digest")?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("the installer has an invalid GitHub SHA-256 digest");
    }
    Ok(digest)
}

fn file_sha256(path: &Path) -> anyhow::Result<String> {
    let mut file = File::open(path).context("unable to read the installer for SHA-256")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .context("unable to calculate the installer SHA-256")?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let hash = hasher.finalize();
    let mut encoded = String::with_capacity(64);
    for byte in hash {
        write!(&mut encoded, "{byte:02x}").expect("writing SHA-256 to a String cannot fail");
    }
    Ok(encoded)
}

fn release_version(tag: &str) -> anyhow::Result<&str> {
    let version = tag.strip_prefix('v').unwrap_or(tag);
    parse_version(version)?;
    Ok(version)
}

fn parse_version(version: &str) -> anyhow::Result<[u64; 3]> {
    let parts = version
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .context("the release version is invalid")?;
    let [major, minor, patch] = parts.as_slice() else {
        bail!("the release version must contain three numeric parts");
    };
    Ok([*major, *minor, *patch])
}

fn is_newer_version(latest: &str, current: &str) -> anyhow::Result<bool> {
    Ok(parse_version(latest)? > parse_version(current)?)
}

fn display_error(error: anyhow::Error) -> String {
    format!("{error:#}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PUBLIC_KEY: &str = "untrusted comment: minisign public key E7620F1842B4E81F\nRWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";
    const TEST_SIGNATURE: &str = "untrusted comment: signature from minisign secret key\nRUQf6LRCGA9i559r3g7V1qNyJDApGip8MfqcadIgT9CuhV3EMhHoN1mGTkUidF/z7SrlQgXdy8ofjb7bNJJylDOocrCo8KLzZwo=\ntrusted comment: timestamp:1556193335\tfile:test\ny/rUw2y8/hOUYjZU71eHp/Wo1KZ40fGy2VJEDl34XMJM+TX48Ss/17u3IvIfbVR1FkZZSNCisQbuQY+bHwhEBg==";

    fn release(url: &str) -> GitHubRelease {
        GitHubRelease {
            tag_name: "v1.1.23".into(),
            assets: vec![
                GitHubAsset {
                    name: "Relay_1.1.23_x64-setup.exe".into(),
                    browser_download_url: url.into(),
                    size: 7_472_679,
                    digest: Some(format!("sha256:{}", "a".repeat(64))),
                },
                GitHubAsset {
                    name: "Relay_1.1.23_x64-setup.exe.sig".into(),
                    browser_download_url: format!("{url}.sig"),
                    size: 300,
                    digest: None,
                },
            ],
        }
    }

    #[test]
    fn compares_numeric_release_versions() {
        assert!(is_newer_version("1.1.23", "1.1.22").unwrap());
        assert!(is_newer_version("1.2.0", "1.1.99").unwrap());
        assert!(!is_newer_version("1.1.22", "1.1.22").unwrap());
        assert!(!is_newer_version("1.1.21", "1.1.22").unwrap());
        assert!(is_newer_version("1.1-beta", "1.1.22").is_err());
    }

    #[test]
    fn accepts_only_the_expected_official_installer_url() {
        let official = release(
            "https://github.com/stealthsrc/relay/releases/download/v1.1.23/Relay_1.1.23_x64-setup.exe",
        );
        let asset = installer_asset(&official, "1.1.23").unwrap();
        assert!(validated_download_url(&official, asset, "1.1.23").is_ok());

        let foreign = release(
            "https://example.com/stealthsrc/relay/releases/download/v1.1.23/Relay_1.1.23_x64-setup.exe",
        );
        let asset = installer_asset(&foreign, "1.1.23").unwrap();
        assert!(validated_download_url(&foreign, asset, "1.1.23").is_err());
    }

    #[test]
    fn requires_a_sha256_digest_from_github() {
        let mut release = release(
            "https://github.com/stealthsrc/relay/releases/download/v1.1.23/Relay_1.1.23_x64-setup.exe",
        );
        assert!(expected_digest(&release.assets[0]).is_ok());
        release.assets[0].digest = None;
        assert!(expected_digest(&release.assets[0]).is_err());
    }

    #[test]
    fn requires_the_expected_official_signature_asset() {
        let mut release = release(
            "https://github.com/stealthsrc/relay/releases/download/v1.1.23/Relay_1.1.23_x64-setup.exe",
        );
        let asset = signature_asset(&release, "1.1.23").unwrap();
        assert!(validated_asset_url(&release, asset, "Relay_1.1.23_x64-setup.exe.sig").is_ok());
        release.assets.pop();
        assert!(signature_asset(&release, "1.1.23").is_err());
    }

    #[test]
    fn verifies_signed_payload_and_rejects_tampering_or_missing_signature() {
        let directory = tempfile::tempdir().unwrap();
        let payload = directory.path().join("update.bin");
        let public_key = BASE64_STANDARD.encode(TEST_PUBLIC_KEY);
        let signature = BASE64_STANDARD.encode(TEST_SIGNATURE);

        fs::write(&payload, b"test").unwrap();
        assert!(verify_file_signature_with_key(&payload, &signature, &public_key, "test").is_ok());

        fs::write(&payload, b"Test").unwrap();
        assert!(verify_file_signature_with_key(&payload, &signature, &public_key, "test").is_err());
        assert!(verify_file_signature_with_key(&payload, "", &public_key, "test").is_err());

        fs::write(&payload, b"test").unwrap();
        assert!(
            verify_file_signature_with_key(
                &payload,
                &signature,
                &public_key,
                "Relay_9.9.9_x64-setup.exe"
            )
            .is_err()
        );
    }

    #[test]
    fn pinned_updater_public_key_is_valid() {
        let decoded = decode_tauri_minisign(UPDATER_PUBLIC_KEY, "updater public key").unwrap();
        assert!(PublicKey::decode(&decoded).is_ok());
    }

    #[test]
    #[ignore = "requires RELAY_SIGNED_INSTALLER from a signed release build"]
    fn signed_release_artifact_matches_pinned_key_and_rejects_tampering() {
        let installer = PathBuf::from(
            env::var_os("RELAY_SIGNED_INSTALLER").expect("RELAY_SIGNED_INSTALLER is required"),
        );
        let mut signature_path = installer.as_os_str().to_owned();
        signature_path.push(".sig");
        let signature = fs::read_to_string(PathBuf::from(signature_path)).unwrap();

        let expected_name = installer.file_name().unwrap().to_str().unwrap();
        verify_file_signature(&installer, &signature, expected_name).unwrap();

        let directory = tempfile::tempdir().unwrap();
        let tampered = directory.path().join("tampered-update.exe");
        fs::copy(&installer, &tampered).unwrap();
        OpenOptions::new()
            .append(true)
            .open(&tampered)
            .unwrap()
            .write_all(b"tampered")
            .unwrap();
        assert!(verify_file_signature(&tampered, &signature, expected_name).is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn verified_installer_lock_allows_hashing_and_launch() {
        let directory = tempfile::tempdir().unwrap();
        let source = PathBuf::from(env::var_os("SystemRoot").unwrap())
            .join("System32")
            .join("where.exe");
        let executable = directory.path().join("Relay_update_test.exe");
        fs::copy(source, &executable).unwrap();

        let mut options = OpenOptions::new();
        options.read(true).share_mode(0x0000_0001);
        let _lock = options.open(&executable).unwrap();

        assert_eq!(file_sha256(&executable).unwrap().len(), 64);
        assert!(Command::new(&executable).arg("cmd.exe").output().is_ok());
    }
}
