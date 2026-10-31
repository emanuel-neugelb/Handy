//! NGLB managed-deployment layer.
//!
//! Policy comes from a Jamf-delivered managed-preferences plist, never from the
//! user-editable settings store. Missing, unreadable or tampered policy fails
//! closed: speech-to-text only, no network.

use crate::settings::{
    AppSettings, PostProcessApiFormat, PostProcessProvider, APPLE_INTELLIGENCE_DEFAULT_MODEL_ID,
    APPLE_INTELLIGENCE_PROVIDER_ID,
};
use log::warn;
use serde::Deserialize;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

const POLICY_PLIST: &str = "/Library/Managed Preferences/com.pais.handy.plist";
const DEFAULT_MODEL_STAGE_DIR: &str = "/Library/Application Support/NGLB/Handy/models";
const CACHE_TTL: Duration = Duration::from_secs(5);

pub const LOCAL_PROVIDER_ID: &str = "nglb_local";
pub const FOUNDRY_PROVIDER_ID: &str = "nglb_foundry";
const LOCAL_BASE_URL: &str = "http://localhost:11434/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmMode {
    Off,
    Local,
    Foundry,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub llm_mode: LlmMode,
    pub foundry_endpoint: Option<reqwest::Url>,
    pub foundry_deployment: String,
    pub model_hosts: Vec<String>,
    pub model_stage_dir: PathBuf,
    pub policy_version: String,
    pub trusted: bool,
}

impl Policy {
    /// Used whenever the managed policy is absent or untrusted.
    pub fn locked_down() -> Self {
        Policy {
            llm_mode: LlmMode::Off,
            foundry_endpoint: None,
            foundry_deployment: String::new(),
            model_hosts: Vec::new(),
            model_stage_dir: PathBuf::from(DEFAULT_MODEL_STAGE_DIR),
            policy_version: "none".to_string(),
            trusted: false,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RawPolicy {
    llm_mode: Option<String>,
    foundry_endpoint: Option<String>,
    foundry_deployment: Option<String>,
    model_hosts: Option<Vec<String>>,
    model_stage_dir: Option<String>,
    policy_version: Option<String>,
}

fn parse_policy(raw: RawPolicy) -> Policy {
    let mut policy = Policy::locked_down();
    policy.trusted = true;
    policy.policy_version = raw.policy_version.unwrap_or_else(|| "unversioned".into());

    let foundry_endpoint = raw
        .foundry_endpoint
        .as_deref()
        .and_then(|u| reqwest::Url::parse(u).ok())
        .filter(|u| u.scheme() == "https" && u.host_str().is_some());

    policy.llm_mode = match raw.llm_mode.as_deref() {
        Some("local") => LlmMode::Local,
        // Foundry without a valid https endpoint degrades to local only.
        Some("foundry") if foundry_endpoint.is_some() => LlmMode::Foundry,
        Some("foundry") => LlmMode::Local,
        _ => LlmMode::Off,
    };
    if policy.llm_mode == LlmMode::Foundry {
        policy.foundry_endpoint = foundry_endpoint;
        policy.foundry_deployment = raw.foundry_deployment.unwrap_or_default();
    }
    policy.model_hosts = raw
        .model_hosts
        .unwrap_or_default()
        .into_iter()
        .map(|h| h.trim().to_ascii_lowercase())
        .filter(|h| !h.is_empty())
        .collect();
    if let Some(dir) = raw.model_stage_dir.filter(|d| !d.is_empty()) {
        policy.model_stage_dir = PathBuf::from(dir);
    }
    policy
}

/// Managed-preferences files are written by root via MDM. Anything writable by
/// a non-root user is treated as tampered.
#[cfg(unix)]
fn file_is_trusted(path: &std::path::Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match std::fs::metadata(path) {
        Ok(m) => m.uid() == 0 && (m.mode() & 0o022) == 0,
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn file_is_trusted(_path: &std::path::Path) -> bool {
    false
}

fn load_policy_from(path: &std::path::Path, require_root_owner: bool) -> Policy {
    if require_root_owner && !file_is_trusted(path) {
        if path.exists() {
            warn!("Managed policy {path:?} is not root-owned or is writable; failing closed");
        }
        return Policy::locked_down();
    }
    match plist::from_file::<_, RawPolicy>(path) {
        Ok(raw) => parse_policy(raw),
        Err(e) => {
            warn!("Managed policy unreadable ({e}); failing closed");
            Policy::locked_down()
        }
    }
}

static CACHE: Mutex<Option<(Instant, Policy)>> = Mutex::new(None);

pub fn policy() -> Policy {
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((at, p)) = cache.as_ref() {
        if at.elapsed() < CACHE_TTL {
            return p.clone();
        }
    }
    let p = load_policy_from(std::path::Path::new(POLICY_PLIST), true);
    *cache = Some((Instant::now(), p.clone()));
    p
}

fn is_loopback_host(host: &url::Host<&str>) -> bool {
    match host {
        url::Host::Domain(d) => d.eq_ignore_ascii_case("localhost"),
        url::Host::Ipv4(ip) => IpAddr::V4(*ip).is_loopback(),
        url::Host::Ipv6(ip) => IpAddr::V6(*ip).is_loopback(),
    }
}

fn check_llm_url_with(policy: &Policy, raw: &str) -> Result<(), String> {
    if policy.llm_mode == LlmMode::Off {
        return Err("LLM post-processing is disabled by NGLB policy".into());
    }
    let url = reqwest::Url::parse(raw).map_err(|_| "Invalid URL".to_string())?;
    let host = url.host().ok_or("URL has no host")?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err("Credentials in URL are not allowed".into());
    }
    if is_loopback_host(&host) && matches!(url.scheme(), "http" | "https") {
        return Ok(());
    }
    if policy.llm_mode == LlmMode::Foundry && url.scheme() == "https" {
        if let Some(ep) = &policy.foundry_endpoint {
            let same_host = ep
                .host_str()
                .zip(url.host_str())
                .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b));
            if same_host && ep.port_or_known_default() == url.port_or_known_default() {
                return Ok(());
            }
        }
    }
    Err("Endpoint is not permitted by NGLB policy".into())
}

/// Lets other modules' tests talk to local mock servers; no effect in release.
#[cfg(test)]
fn test_loopback(raw: &str) -> bool {
    reqwest::Url::parse(raw)
        .ok()
        .and_then(|u| u.host().map(|h| is_loopback_host(&h)))
        .unwrap_or(false)
}

/// Hard gate for every LLM request.
pub fn guard_llm_url(raw: &str) -> Result<(), String> {
    #[cfg(test)]
    if test_loopback(raw) {
        return Ok(());
    }
    check_llm_url_with(&policy(), raw)
}

fn check_download_url_with(policy: &Policy, raw: &str) -> Result<(), String> {
    let url = reqwest::Url::parse(raw).map_err(|_| "Invalid URL".to_string())?;
    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if url.scheme() == "https" && policy.model_hosts.iter().any(|h| *h == host) {
        Ok(())
    } else {
        Err(format!(
            "Downloads from '{host}' are not permitted by NGLB policy; models are provisioned by IT"
        ))
    }
}

/// Hard gate for every model download. Empty `modelHosts` disables downloads.
pub fn guard_download_url(raw: &str) -> Result<(), String> {
    #[cfg(test)]
    if test_loopback(raw) {
        return Ok(());
    }
    check_download_url_with(&policy(), raw)
}

/// Directory of IT-provisioned models (read-only, root-owned).
pub fn model_stage_dir() -> PathBuf {
    policy().model_stage_dir
}

/// Link IT-provisioned models (root-owned, outside the user profile) into the
/// per-user models dir so no download is needed. Dangling links are removed.
pub fn link_staged_models(models_dir: &std::path::Path) {
    link_staged_models_from(&model_stage_dir(), models_dir);
}

fn link_staged_models_from(stage: &std::path::Path, models_dir: &std::path::Path) {
    #[cfg(unix)]
    {
        if let Ok(entries) = std::fs::read_dir(models_dir) {
            for e in entries.flatten() {
                let p = e.path();
                let dangling = std::fs::symlink_metadata(&p)
                    .map(|m| m.file_type().is_symlink())
                    .unwrap_or(false)
                    && !p.exists();
                if dangling {
                    let _ = std::fs::remove_file(&p);
                }
            }
        }
        let Ok(entries) = std::fs::read_dir(stage) else {
            return;
        };
        for e in entries.flatten() {
            let name = e.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue;
            }
            let dest = models_dir.join(&name);
            if std::fs::symlink_metadata(&dest).is_err() {
                if let Err(err) = std::os::unix::fs::symlink(e.path(), &dest) {
                    warn!("Could not link staged model {name:?}: {err}");
                }
            }
        }
    }
    #[cfg(not(unix))]
    let _ = (stage, models_dir);
}

/// Overlay managed values on user settings. Applied on every settings read, so
/// hand edits to the store cannot change a locked value.
pub fn enforce_settings(settings: &mut AppSettings) {
    enforce_settings_with(&policy(), settings);
}

fn enforce_settings_with(policy: &Policy, settings: &mut AppSettings) {
    settings.update_checks_enabled = false;

    let mut providers: Vec<PostProcessProvider> = Vec::new();
    if policy.llm_mode != LlmMode::Off {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        providers.push(PostProcessProvider {
            id: APPLE_INTELLIGENCE_PROVIDER_ID.to_string(),
            label: "Apple Intelligence (on-device)".to_string(),
            base_url: "apple-intelligence://local".to_string(),
            allow_base_url_edit: false,
            models_endpoint: None,
            supports_structured_output: true,
            api_format: PostProcessApiFormat::OpenAi,
        });
        providers.push(PostProcessProvider {
            id: LOCAL_PROVIDER_ID.to_string(),
            label: "Local model (Ollama)".to_string(),
            base_url: LOCAL_BASE_URL.to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
            supports_structured_output: false,
            api_format: PostProcessApiFormat::OpenAi,
        });
        if let (LlmMode::Foundry, Some(ep)) = (policy.llm_mode, &policy.foundry_endpoint) {
            providers.push(PostProcessProvider {
                id: FOUNDRY_PROVIDER_ID.to_string(),
                label: "NGLB Azure AI Foundry".to_string(),
                base_url: ep.as_str().trim_end_matches('/').to_string(),
                allow_base_url_edit: false,
                models_endpoint: Some("/models".to_string()),
                supports_structured_output: true,
                api_format: PostProcessApiFormat::OpenAi,
            });
            settings.post_process_models.insert(
                FOUNDRY_PROVIDER_ID.to_string(),
                policy.foundry_deployment.clone(),
            );
        }
    }

    settings.post_process_providers = providers;
    let selected_ok = settings
        .post_process_providers
        .iter()
        .any(|p| p.id == settings.post_process_provider_id);
    if !selected_ok {
        settings.post_process_provider_id = settings
            .post_process_providers
            .first()
            .map(|p| p.id.clone())
            .unwrap_or_default();
    }
    if settings.post_process_providers.is_empty() {
        settings.post_process_enabled = false;
    }
    if settings.post_process_provider_id == APPLE_INTELLIGENCE_PROVIDER_ID {
        settings
            .post_process_models
            .entry(APPLE_INTELLIGENCE_PROVIDER_ID.to_string())
            .or_insert_with(|| APPLE_INTELLIGENCE_DEFAULT_MODEL_ID.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy_with(mode: &str, endpoint: Option<&str>) -> Policy {
        parse_policy(RawPolicy {
            llm_mode: Some(mode.into()),
            foundry_endpoint: endpoint.map(String::from),
            foundry_deployment: Some("gpt-nglb".into()),
            model_hosts: Some(vec!["Models.Internal.example".into()]),
            ..Default::default()
        })
    }

    #[test]
    fn locked_down_blocks_everything() {
        let p = Policy::locked_down();
        for u in [
            "http://localhost:11434/v1",
            "https://api.openai.com/v1",
            "https://foundry.example.com",
        ] {
            assert!(check_llm_url_with(&p, u).is_err(), "{u}");
        }
        assert!(check_download_url_with(&p, "https://huggingface.co/x").is_err());
    }

    #[test]
    fn local_mode_allows_only_loopback() {
        let p = policy_with("local", None);
        for u in [
            "http://localhost:11434/v1",
            "http://127.0.0.1:11434/v1/models",
            "http://[::1]:11434/v1",
            "http://LOCALHOST/v1",
            "http://2130706433/v1",
            "http://0x7f.1/v1",
        ] {
            assert!(check_llm_url_with(&p, u).is_ok(), "{u}");
        }
        for u in [
            "https://api.openai.com/v1",
            "https://api.anthropic.com/v1",
            "https://openrouter.ai/api/v1",
            "http://localhost.evil.com/v1",
            "http://127.0.0.1.evil.com/v1",
            "http://evil.com@localhost/v1",
            "http://localhost@evil.com/v1",
            "http://169.254.169.254/",
            "http://10.0.0.5/v1",
            "ftp://localhost/",
            "not a url",
        ] {
            assert!(check_llm_url_with(&p, u).is_err(), "{u}");
        }
    }

    #[test]
    fn foundry_mode_allows_only_its_host_over_https() {
        let p = policy_with("foundry", Some("https://nglb-foundry.openai.azure.com/openai/v1"));
        assert!(check_llm_url_with(&p, "https://nglb-foundry.openai.azure.com/openai/v1/chat").is_ok());
        assert!(check_llm_url_with(&p, "https://NGLB-Foundry.openai.azure.com/x").is_ok());
        assert!(check_llm_url_with(&p, "http://nglb-foundry.openai.azure.com/x").is_err());
        assert!(check_llm_url_with(&p, "https://nglb-foundry.openai.azure.com:8443/x").is_err());
        assert!(check_llm_url_with(&p, "https://other.openai.azure.com/x").is_err());
        assert!(check_llm_url_with(&p, "https://nglb-foundry.openai.azure.com.evil.com/x").is_err());
        assert!(check_llm_url_with(&p, "http://localhost:11434/v1").is_ok());
    }

    #[test]
    fn foundry_without_valid_https_endpoint_degrades_to_local() {
        assert_eq!(policy_with("foundry", None).llm_mode, LlmMode::Local);
        assert_eq!(
            policy_with("foundry", Some("http://foundry.example.com")).llm_mode,
            LlmMode::Local
        );
        assert_eq!(policy_with("bogus", None).llm_mode, LlmMode::Off);
    }

    #[test]
    fn downloads_require_listed_https_host() {
        let p = policy_with("off", None);
        assert!(check_download_url_with(&p, "https://models.internal.example/a.bin").is_ok());
        assert!(check_download_url_with(&p, "http://models.internal.example/a.bin").is_err());
        assert!(check_download_url_with(&p, "https://huggingface.co/a.bin").is_err());
        assert!(check_download_url_with(&p, "https://blob.handy.computer/a.bin").is_err());
    }

    #[test]
    fn enforce_replaces_user_providers_and_disables_updates() {
        let mut s = crate::settings::get_default_settings();
        s.post_process_enabled = true;
        s.post_process_provider_id = "openai".into();
        s.update_checks_enabled = true;

        enforce_settings_with(&policy_with("local", None), &mut s);
        assert!(!s.update_checks_enabled);
        assert!(s.post_process_providers.iter().all(|p| {
            p.id == LOCAL_PROVIDER_ID || p.id == APPLE_INTELLIGENCE_PROVIDER_ID
        }));
        assert!(s.post_process_providers.iter().all(|p| !p.allow_base_url_edit));
        assert_ne!(s.post_process_provider_id, "openai");
    }

    #[test]
    fn enforce_off_disables_post_processing() {
        let mut s = crate::settings::get_default_settings();
        s.post_process_enabled = true;
        enforce_settings_with(&Policy::locked_down(), &mut s);
        assert!(s.post_process_providers.is_empty());
        assert!(!s.post_process_enabled);
    }

    #[test]
    fn staged_models_are_linked_and_dangling_links_removed() {
        let base = std::env::temp_dir().join(format!("nglb-stage-{}", std::process::id()));
        let (stage, models) = (base.join("stage"), base.join("models"));
        std::fs::create_dir_all(&stage).unwrap();
        std::fs::create_dir_all(&models).unwrap();
        std::fs::write(stage.join("ggml-small.bin"), b"x").unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(base.join("gone"), models.join("old.bin")).unwrap();
        }
        link_staged_models_from(&stage, &models);
        assert!(models.join("ggml-small.bin").exists());
        assert!(std::fs::symlink_metadata(models.join("old.bin")).is_err());
        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn plist_roundtrip_and_untrusted_file_fails_closed() {
        let dir = std::env::temp_dir().join(format!("nglb-policy-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("policy.plist");
        std::fs::write(
            &path,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>llmMode</key><string>local</string>
<key>policyVersion</key><string>7</string>
</dict></plist>"#,
        )
        .unwrap();
        let p = load_policy_from(&path, false);
        assert_eq!(p.llm_mode, LlmMode::Local);
        assert_eq!(p.policy_version, "7");
        // A user-owned file is never trusted when ownership is required.
        let p = load_policy_from(&path, true);
        assert_eq!(p.llm_mode, LlmMode::Off);
        assert!(!p.trusted);
        std::fs::remove_dir_all(&dir).ok();
    }
}
