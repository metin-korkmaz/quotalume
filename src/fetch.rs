use std::sync::RwLock;

use reqwest::Client;

use crate::config::{claude_credentials_path, codex_auth_path, Config};
use crate::credentials::{parse_claude_credentials, parse_codex_auth};
use crate::model::{ProviderId, ProviderSnapshot};
use crate::providers::{claude, codex, ollama, openrouter};

pub async fn fetch_all(config: &Config) -> Vec<ProviderSnapshot> {
    let mut snapshots = Vec::with_capacity(4);
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap_or_default();

    // Codex (OAuth)
    snapshots.push(fetch_codex(&client).await);

    // Claude Code (OAuth)
    snapshots.push(fetch_claude(&client).await);

    // OpenRouter
    snapshots.push(fetch_openrouter(&client, config).await);

    // Ollama Cloud
    snapshots.push(fetch_ollama(&client, config).await);

    snapshots
}

async fn fetch_codex(client: &Client) -> ProviderSnapshot {
    let path = codex_auth_path();
    if !path.exists() {
        return ProviderSnapshot::not_configured(
            ProviderId::Codex,
            "codex ile oturum açın: ~/.codex/auth.json",
        );
    }
    match std::fs::read_to_string(&path) {
        Ok(raw) => match parse_codex_auth(&raw) {
            Ok(cred) => {
                let url = "https://chatgpt.com/backend-api/wham/usage";
                let mut req = client.get(url);
                req = req
                    .bearer_auth(&cred.access_token)
                    .header("Accept", "application/json")
                    .header("User-Agent", "QuotaLume/0.1");
                if let Some(ref acct) = cred.account_id {
                    req = req.header("ChatGPT-Account-Id", acct);
                }
                match req.send().await {
                    Ok(resp) if resp.status().is_success() => {
                        match resp.text().await {
                            Ok(body) => codex::parse_usage(&body).unwrap_or_else(|e| {
                                ProviderSnapshot::unavailable(
                                    ProviderId::Codex,
                                    format!("Codex yanıt parse edilemedi: {e}"),
                                )
                            }),
                            Err(e) => ProviderSnapshot::unavailable(
                                ProviderId::Codex,
                                format!("Codex yanıt okunamadı: {e}"),
                            ),
                        }
                    }
                    Ok(resp) if resp.status() == 401 || resp.status() == 403 => {
                        ProviderSnapshot::unavailable(
                            ProviderId::Codex,
                            "Codex OAuth token süresi dolmuş. `codex` ile yeniden giriş yapın.",
                        )
                    }
                    Ok(resp) => ProviderSnapshot::unavailable(
                        ProviderId::Codex,
                        format!("Codex API HTTP {}", resp.status()),
                    ),
                    Err(e) => ProviderSnapshot::unavailable(
                        ProviderId::Codex,
                        format!("Codex API ağ hatası: {e}"),
                    ),
                }
            }
            Err(e) => ProviderSnapshot::unavailable(
                ProviderId::Codex,
                format!("Codex auth.json parse edilemedi: {e}"),
            ),
        },
        Err(e) => ProviderSnapshot::unavailable(
            ProviderId::Codex,
            format!("Codex auth.json okunamadı: {e}"),
        ),
    }
}

async fn fetch_claude(client: &Client) -> ProviderSnapshot {
    let path = claude_credentials_path();
    if !path.exists() {
        return ProviderSnapshot::not_configured(
            ProviderId::Claude,
            "claude ile oturum açın: ~/.claude/.credentials.json",
        );
    }
    match std::fs::read_to_string(&path) {
        Ok(raw) => match parse_claude_credentials(&raw) {
            Ok(cred) => {
                if !cred.has_usage_scope() {
                    return ProviderSnapshot::unavailable(
                        ProviderId::Claude,
                        "Claude OAuth token'ında user:profile yetkisi yok.",
                    );
                }
                let url = "https://api.anthropic.com/api/oauth/usage";
                let plan = cred
                    .subscription_type
                    .as_deref()
                    .map(|s| s.to_string());
                let req = client
                    .get(url)
                    .bearer_auth(&cred.access_token)
                    .header("Accept", "application/json")
                    .header("anthropic-beta", "oauth-2025-04-20")
                    .header("User-Agent", "claude-code/2.1.0");
                match req.send().await {
                    Ok(resp) if resp.status().is_success() => {
                        match resp.text().await {
                            Ok(body) => claude::parse_usage(&body, plan.as_deref())
                                .unwrap_or_else(|e| ProviderSnapshot::unavailable(
                                    ProviderId::Claude,
                                    format!("Claude yanıt parse edilemedi: {e}"),
                                )),
                            Err(e) => ProviderSnapshot::unavailable(
                                ProviderId::Claude,
                                format!("Claude yanıt okunamadı: {e}"),
                            ),
                        }
                    }
                    Ok(resp) if resp.status() == 401 || resp.status() == 403 => {
                        ProviderSnapshot::unavailable(
                            ProviderId::Claude,
                            "Claude OAuth token süresi dolmuş. `claude` ile yeniden giriş yapın.",
                        )
                    }
                    Ok(resp) => ProviderSnapshot::unavailable(
                        ProviderId::Claude,
                        format!("Claude API HTTP {}", resp.status()),
                    ),
                    Err(e) => ProviderSnapshot::unavailable(
                        ProviderId::Claude,
                        format!("Claude API ağ hatası: {e}"),
                    ),
                }
            }
            Err(e) => ProviderSnapshot::unavailable(
                ProviderId::Claude,
                format!("Claude credentials parse edilemedi: {e}"),
            ),
        },
        Err(e) => ProviderSnapshot::unavailable(
            ProviderId::Claude,
            format!("Claude credentials okunamadı: {e}"),
        ),
    }
}

async fn fetch_openrouter(client: &Client, config: &Config) -> ProviderSnapshot {
    let api_key = match &config.openrouter_api_key {
        Some(key) => key.clone(),
        None => {
            return ProviderSnapshot::not_configured(
                ProviderId::OpenRouter,
                "OPENROUTER_API_KEY ortam değişkeni veya config.toml ayarlayın",
            )
        }
    };

    let credits_url = "https://openrouter.ai/api/v1/credits";
    let key_url = "https://openrouter.ai/api/v1/key";

    let credits_body = match client
        .get(credits_url)
        .bearer_auth(&api_key)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => resp.text().await.ok(),
        Ok(resp) => {
            return ProviderSnapshot::unavailable(
                ProviderId::OpenRouter,
                format!("OpenRouter credits API HTTP {}", resp.status()),
            )
        }
        Err(e) => {
            return ProviderSnapshot::unavailable(
                ProviderId::OpenRouter,
                format!("OpenRouter ağ hatası: {e}"),
            )
        }
    };

    let key_body: Option<String> = match client
        .get(key_url)
        .bearer_auth(&api_key)
        .header("Accept", "application/json")
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r.text().await.ok(),
        _ => None,
    };

    match credits_body {
        Some(body) => openrouter::parse_credits_and_key(&body, key_body.as_deref())
            .unwrap_or_else(|e| ProviderSnapshot::unavailable(
                ProviderId::OpenRouter,
                format!("OpenRouter yanıt parse edilemedi: {e}"),
            )),
        None => ProviderSnapshot::unavailable(
            ProviderId::OpenRouter,
            "OpenRouter kredi yanıtı boş",
        ),
    }
}

async fn fetch_ollama(client: &Client, config: &Config) -> ProviderSnapshot {
    // Try cookie-based HTML scrape first (gives quota windows)
    if let Some(cookie) = &config.ollama_cookie_header {
        let resp = client
            .get("https://ollama.com/settings")
            .header("Cookie", cookie)
            .header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/125.0.0.0 Safari/537.36",
            )
            .header("Accept", "text/html")
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                if let Ok(html) = r.text().await {
                    return ollama::parse_settings_html(&html).unwrap_or_else(|e| {
                        ProviderSnapshot::unavailable(
                            ProviderId::Ollama,
                            format!("Ollama HTML parse hatası: {e}"),
                        )
                    });
                }
            }
            Ok(r) if r.status() == 401 || r.status() == 403 => {
                return ProviderSnapshot::unavailable(
                    ProviderId::Ollama,
                    "Ollama oturum çerezi süresi dolmuş. ollama.com/signin adresinde yeniden giriş yapın.",
                );
            }
            _ => {}
        }
    }

    // Fall back to API key validation (no quota windows)
    if let Some(api_key) = &config.ollama_api_key {
        let resp = client
            .get("https://ollama.com/api/tags")
            .bearer_auth(api_key)
            .header("Accept", "application/json")
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                if let Ok(body) = r.text().await {
                    return ollama::parse_api_key_validation(&body).unwrap_or_else(|e| {
                        ProviderSnapshot::unavailable(
                            ProviderId::Ollama,
                            format!("Ollama API parse hatası: {e}"),
                        )
                    });
                }
            }
            Ok(r) if r.status() == 401 || r.status() == 403 => {
                return ProviderSnapshot::unavailable(
                    ProviderId::Ollama,
                    "Ollama API anahtarı geçersiz veya iptal edilmiş.",
                );
            }
            _ => {}
        }
    }

    ProviderSnapshot::not_configured(
        ProviderId::Ollama,
        "OLLAMA_API_KEY veya OLLAMA_COOKIE ortam değişkeni ayarlayın",
    )
}

pub type SnapshotStore = RwLock<Vec<ProviderSnapshot>>;