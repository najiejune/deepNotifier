use serde::Serialize;
use tauri::State;

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
                if t.enabled {
                    return true;
                }
                // When specific CLI IDs are requested, include them even if not globally enabled
                if let Some(ref ids) = cli_ids {
                    return ids.contains(&t.id);
                }
                false
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

        let result = injector::uninstall_hooks(meta, &cwd);
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
