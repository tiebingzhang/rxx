use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub user_id: String,
    pub server_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hooks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_received: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        println!("Config saved to {:?}", path);
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".rxx.conf"))
    }

    pub fn execute_file_received_hook(&self, sender_id: &str, filename: &str, file_size: u64) {
        if let Some(hooks) = &self.hooks {
            if let Some(hook_cmd) = &hooks.file_received {
                let cmd = hook_cmd.clone();
                let sender = sender_id.to_string();
                let fname = filename.to_string();

                tokio::spawn(async move {
                    println!("DEBUG [HOOK]: Executing file-received hook: {}", cmd);

                    let result = tokio::time::timeout(
                        Duration::from_secs(10),
                        Command::new("sh")
                            .arg("-c")
                            .arg(&cmd)
                            .arg(&sender)
                            .arg(&fname)
                            .arg(file_size.to_string())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .output(),
                    )
                    .await;

                    match result {
                        Ok(Ok(output)) => {
                            if output.status.success() {
                                println!("DEBUG [HOOK]: Hook executed successfully");
                            } else {
                                eprintln!(
                                    "WARN [HOOK]: Hook failed with exit code: {:?}",
                                    output.status.code()
                                );
                                if !output.stderr.is_empty() {
                                    eprintln!(
                                        "WARN [HOOK]: stderr: {}",
                                        String::from_utf8_lossy(&output.stderr)
                                    );
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            eprintln!("WARN [HOOK]: Failed to execute hook: {}", e);
                        }
                        Err(_) => {
                            eprintln!("WARN [HOOK]: Hook execution timed out after 10 seconds");
                        }
                    }
                });
            }
        }
    }
}
