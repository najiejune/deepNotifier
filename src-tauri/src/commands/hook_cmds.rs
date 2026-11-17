use serde::Serialize;
use tauri::{Emitter, State};

use crate::config::schema::HookStatus;
use crate::hook::cli_configs::{all_cli_metas, check_cli_installed};
use crate::hook::injector::{self, HookInstallResult};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CliStatus {
    pub cli_id: String,
    pub cli_installed: bool,
    pub hook_installed: HookStatus,
}

#[tauri::command]
pub async fn install_hooks(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    cli_ids: Option<Vec<String>>,
) -> Result<Vec<HookInstallResult>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    let (enabled, port, tools, approval_timeout_enabled, approval_timeout_secs): (bool, u16, Vec<_>, bool, u32) = {
        let cfg = state.config.read().await;
        let tools: Vec<_> = cfg
            .hook
            .cli_tools
            .iter()
            .filter(|t| {
                // Specific CLI IDs requested: strict filter, install exactly
                // those — otherwise a single-CLI install would silently drag
                // every globally enabled tool along (e.g. triggering the
                // deepseek bridge install when installing another CLI).
                if let Some(ref ids) = cli_ids {
                    return ids.contains(&t.id);
                }
                t.enabled
            })
            .cloned()
            .collect();
        (cfg.hook.enabled, cfg.webhook.port, tools, cfg.hook.approval_timeout_enabled, cfg.hook.approval_timeout_secs)
    };

    if !enabled {
        return Err("Hook system is disabled. Enable it in settings first.".into());
    }

    let metas = all_cli_metas();
    let mut results = Vec::new();

    for tool in &tools {
        let meta = match metas.iter().find(|m| m.id == tool.id) {
            Some(m) => m,
            None => {
                results.push(HookInstallResult {
                    cli_id: tool.id.clone(),
                    success: false,
                    message: format!("Unknown CLI: {}", tool.id),
                    config_path: String::new(),
                    events_injected: vec![],
                });
                continue;
            }
        };

        // Check if the CLI is actually installed first
        if !check_cli_installed(meta, &cwd) {
            results.push(HookInstallResult {
                cli_id: tool.id.clone(),
                success: false,
                message: format!("{} CLI not found. Install {} first.", meta.name, meta.name),
                config_path: String::new(),
                events_injected: vec![],
            });
            continue;
        }

        // DeepSeek Harness: the bridge package must be a profile dependency
        // BEFORE cordis.patch.yml references it — a mounted but unresolvable
        // plugin makes dsh fail to boot. Install the bridge into every
        // profile first and only write the patch mount when that fully
        // succeeds. Blocking subprocess calls run on the blocking thread
        // pool to keep the runtime free.
        if meta.id == "deepseek" {
            let home = dirs::home_dir().unwrap_or_default();
            let progress_app = app.clone();
            let outcome = match tokio::task::spawn_blocking(move || {
                crate::hook::dsh_plugin::ensure_bridge_installed(&home, &|msg: &str| {
                    let _ = progress_app.emit(
                        "hook-install-progress",
                        serde_json::json!({ "cli_id": "deepseek", "message": msg }),
                    );
                })
            })
            .await
            {
                Ok(o) => o,
                Err(e) => {
                    results.push(HookInstallResult {
                        cli_id: tool.id.clone(),
                        success: false,
                        message: format!("bridge install task failed: {}", e),
                        config_path: String::new(),
                        events_injected: vec![],
                    });
                    continue;
                }
            };
            let safe_to_mount = outcome.runner.is_some()
                && !outcome.profiles.is_empty()
                && outcome.all_succeeded();
            if !safe_to_mount {
                results.push(HookInstallResult {
                    cli_id: tool.id.clone(),
                    success: false,
                    message: format!(
                        "bridge plugin not ready, hook mount skipped (mounting without the package would break dsh boot){}",
                        outcome.summary("add")
                    ),
                    config_path: String::new(),
                    events_injected: vec![],
                });
                continue;
            }
            let mut result = injector::install_hooks(
                meta,
                tool,
                port,
                &cwd,
                approval_timeout_enabled,
                approval_timeout_secs,
            );
            result.message.push_str(&outcome.summary("add"));
            results.push(result);
            continue;
        }

        let result = injector::install_hooks(meta, tool, port, &cwd, approval_timeout_enabled, approval_timeout_secs);
        results.push(result);
    }

    // Update install_status for each tool
    {
        let mut cfg = state.config.write().await;
        for result in &results {
            if let Some(tool) = cfg.hook.cli_tools.iter_mut().find(|t| t.id == result.cli_id) {
                if result.success {
                    tool.install_status = HookStatus::Installed;
                    if !result.config_path.is_empty() {
                        tool.config_path = Some(result.config_path.clone());
                    }
                } else {
                    tool.install_status = HookStatus::Error(result.message.clone());
                }
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn uninstall_hooks(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    cli_ids: Option<Vec<String>>,
) -> Result<Vec<HookInstallResult>, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;

    let tool_ids: Vec<String> = match &cli_ids {
        Some(ids) if !ids.is_empty() => ids.clone(),
        _ => {
            let cfg = state.config.read().await;
            cfg.hook
                .cli_tools
                .iter()
                .map(|t| t.id.clone())
                .collect()
        }
    };

    let metas = all_cli_metas();
    let mut results = Vec::new();

    for cli_id in &tool_ids {
        let meta = match metas.iter().find(|m| m.id == *cli_id) {
            Some(m) => m,
            None => {
                results.push(HookInstallResult {
                    cli_id: cli_id.clone(),
                    success: false,
                    message: format!("Unknown CLI: {}", cli_id),
                    config_path: String::new(),
                    events_injected: vec![],
                });
                continue;
            }
        };

        let mut result = injector::uninstall_hooks(meta, &cwd);

        // DeepSeek Harness: also remove the bridge package from every
        // profile, mirroring the install path.
        if result.success && meta.id == "deepseek" {
            let home = dirs::home_dir().unwrap_or_default();
            let progress_app = app.clone();
            match tokio::task::spawn_blocking(move || {
                crate::hook::dsh_plugin::uninstall_bridge(&home, &|msg: &str| {
                    let _ = progress_app.emit(
                        "hook-install-progress",
                        serde_json::json!({ "cli_id": "deepseek", "message": msg }),
                    );
                })
            })
            .await
            {
                Ok(outcome) => result.message.push_str(&outcome.summary("remove")),
                Err(e) => result
                    .message
                    .push_str(&format!("; bridge removal task failed: {}", e)),
            }
        }

        results.push(result);
    }

    // Update install_status
    {
        let mut cfg = state.config.write().await;
        for result in &results {
            if let Some(tool) = cfg.hook.cli_tools.iter_mut().find(|t| t.id == result.cli_id) {
                if result.success {
                    tool.install_status = HookStatus::NotInstalled;
                    tool.config_path = None;
                }
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn check_cli_status(
    state: State<'_, AppState>,
) -> Result<Vec<CliStatus>, String> {
    let cfg = state.config.read().await;
    let cli_installed_map = state.cli_installed_cache.read().await.clone();

    let mut statuses = Vec::new();
    for tool in &cfg.hook.cli_tools {
        let cli_installed = cli_installed_map
            .get(&tool.id)
            .copied()
            .unwrap_or(false);

        statuses.push(CliStatus {
            cli_id: tool.id.clone(),
            cli_installed,
            hook_installed: tool.install_status.clone(),
        });
    }

    Ok(statuses)
}
