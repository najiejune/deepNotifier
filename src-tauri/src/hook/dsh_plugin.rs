//! DeepSeek Harness (dsh) bridge plugin installation.
//!
//! The hook bridge `@deepseek-ai/dsh-hooks-claude-code` is an official but
//! separately published npm package: mounting it in `cordis.patch.yml` only
//! works once the package is a dependency of the profile. dsh's own plugin
//! manager installs it per profile via `dsh plugin --profile <name> add <pkg>`
//! (which forwards to pnpm in the profile directory).
//!
//! Many users never install a global `dsh` binary and instead run
//! `npx @deepseek-ai/dsh web`; in that case every invocation goes through
//! npx. Note that `dsh plugin` requires `pnpm` on PATH either way.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub(crate) const DSH_BRIDGE_PLUGIN: &str = "@deepseek-ai/dsh-hooks-claude-code";
/// Peer dependency the bridge imports at module top level. It is NOT part of
/// the npm-published dsh runtime tree, and profile workspaces set
/// `autoInstallPeers: false`, so it must be installed alongside the bridge or
/// the bridge fails to activate ("Cannot find package
/// '@deepseek-ai/dsh-hook-protocol'").
const DSH_HOOK_PROTOCOL: &str = "@deepseek-ai/dsh-hook-protocol";
const DSH_PACKAGE: &str = "@deepseek-ai/dsh";

/// `dsh plugin add` forwards to pnpm; direct dsh installs are quick, but an
/// npx cold start downloads the whole dsh package first.
const PLUGIN_OP_TIMEOUT: Duration = Duration::from_secs(180);
const PLUGIN_OP_TIMEOUT_NPX: Duration = Duration::from_secs(600);
const VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(600);

/// DeepSeek Harness data root: `$DSH_HOME`, defaulting to `~/.dsh`.
pub(crate) fn dsh_home(home_dir: &Path) -> PathBuf {
    std::env::var_os("DSH_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir.join(".dsh"))
}

/// How the dsh CLI can be invoked on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DshRunner {
    /// A `dsh`/`dsh-tui` binary (or npm shim) is on PATH.
    Direct,
    /// No binary on PATH; run via `npx -y @deepseek-ai/dsh`.
    Npx,
}

impl DshRunner {
    /// Base command prefix, e.g. `["dsh"]` or `["npx", "-y", "@deepseek-ai/dsh"]`.
    fn argv(self) -> &'static [&'static str] {
        match self {
            DshRunner::Direct => &["dsh"],
            DshRunner::Npx => &["npx", "-y", DSH_PACKAGE],
        }
    }

    fn timeout(self) -> Duration {
        match self {
            DshRunner::Direct => PLUGIN_OP_TIMEOUT,
            DshRunner::Npx => PLUGIN_OP_TIMEOUT_NPX,
        }
    }

    /// Display form for hints, e.g. `npx -y @deepseek-ai/dsh`.
    fn display(self) -> String {
        self.argv().join(" ")
    }
}

/// Probe for a binary on PATH (Windows resolves .cmd shims via `where`).
fn on_path(bin: &str) -> bool {
    let mut cmd = if cfg!(windows) {
        Command::new("where")
    } else {
        Command::new("which")
    };
    cmd.arg(bin);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

/// Detect how to invoke dsh: prefer a real binary, fall back to npx.
pub fn detect_runner() -> Option<DshRunner> {
    for bin in ["dsh", "dsh.exe", "dsh-tui", "dsh-tui.exe"] {
        if on_path(bin) {
            return Some(DshRunner::Direct);
        }
    }
    if on_path("npx") {
        return Some(DshRunner::Npx);
    }
    None
}

/// The manual command users can run when automatic install/remove is skipped.
fn manual_hint(runner: Option<DshRunner>, op: &str, profile: &str) -> String {
    let base = runner
        .map(|r| r.display())
        .unwrap_or_else(|| "dsh".to_string());
    format!(
        "run `{} plugin --profile {} {} {} {}` manually",
        base, profile, op, DSH_BRIDGE_PLUGIN, DSH_HOOK_PROTOCOL
    )
}

/// Outcome of installing/removing the hook bridge across dsh profiles.
#[derive(Debug, Default)]
pub struct BridgeInstallOutcome {
    /// How dsh can be invoked, if at all.
    pub runner: Option<DshRunner>,
    /// Detected dsh version (e.g. "0.2.2-rc.1"), when probeable.
    pub dsh_version: Option<String>,
    /// Profiles discovered under `$DSH_HOME/profiles/`.
    pub profiles: Vec<String>,
    /// Profiles where `dsh plugin <op>` exited successfully.
    pub installed: Vec<String>,
    /// Profiles where the operation failed (with a short reason).
    pub failed: Vec<(String, String)>,
    /// pnpm was missing and got installed automatically via `npm i -g pnpm`.
    pub pnpm_auto_installed: bool,
}

impl BridgeInstallOutcome {
    /// Every discovered profile succeeded (or there was nothing to do for a
    /// removal). Used by the caller to decide whether writing the patch mount
    /// is safe: a mounted bridge that is missing from any profile's
    /// dependencies makes that profile fail to boot.
    pub fn all_succeeded(&self) -> bool {
        self.failed.is_empty()
    }

    /// Human-readable suffix appended to the hook install/uninstall result
    /// message. `op` is the plugin subcommand used ("add" or "remove").
    pub fn summary(&self, op: &str) -> String {
        if self.runner.is_none() {
            return format!(
                "; bridge plugin not {} (no dsh binary on PATH and npx unavailable) — {}",
                if op == "add" { "installed" } else { "removed" },
                manual_hint(None, op, "<profile>")
            );
        }
        if self.profiles.is_empty() {
            if op == "add" {
                return format!(
                    "; no dsh profiles found yet — start dsh once and reinstall, or {}",
                    manual_hint(self.runner, op, "web")
                );
            }
            return String::new();
        }
        let mut parts = Vec::new();
        if self.pnpm_auto_installed {
            parts.push("pnpm was missing; installed automatically via npm".to_string());
        }
        if !self.installed.is_empty() {
            let verb = if op == "add" { "installed into" } else { "removed from" };
            parts.push(format!(
                "bridge plugin {} profiles: {}",
                verb,
                self.installed.join(", ")
            ));
        }
        for (profile, reason) in &self.failed {
            parts.push(format!(
                "bridge {} failed for profile {} ({}) — {}",
                op,
                profile,
                reason,
                manual_hint(self.runner, op, profile)
            ));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("; {}", parts.join("; "))
        }
    }
}

/// Profiles are directories under `$DSH_HOME/profiles/` holding a
/// package.json or cordis.yml (i.e. already initialised by dsh).
pub fn list_profiles(home_dir: &Path) -> Vec<String> {
    list_profiles_in(&dsh_home(home_dir).join("profiles"))
}

fn list_profiles_in(profiles_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = match std::fs::read_dir(profiles_dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter(|e| {
                let p = e.path();
                p.join("package.json").exists() || p.join("cordis.yml").exists()
            })
            .filter_map(|e| e.file_name().into_string().ok())
            .collect(),
        Err(_) => Vec::new(),
    };
    names.sort();
    names
}

/// Build the process for a dsh invocation. On Windows the npm shims are
/// .cmd files that CreateProcess cannot resolve directly, so go through
/// cmd.exe with the console window suppressed.
fn dsh_command(runner: DshRunner, extra_args: &[&str]) -> Command {
    #[cfg(windows)]
    {
        let mut c = Command::new("cmd");
        c.arg("/c").args(runner.argv()).args(extra_args);
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        c.creation_flags(CREATE_NO_WINDOW);
        c
    }
    #[cfg(not(windows))]
    {
        let mut c = Command::new(runner.argv()[0]);
        c.args(&runner.argv()[1..]).args(extra_args);
        c
    }
}

/// Query `dsh --version`; used to pick a matching bridge plugin version.
fn probe_dsh_version(runner: DshRunner) -> Option<String> {
    let mut child = dsh_command(runner, &["--version"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return None;
                }
                let mut buf = String::new();
                use std::io::Read;
                child.stdout.take()?.read_to_string(&mut buf).ok()?;
                let v = buf.trim().to_string();
                // Expect a bare semver-ish token like "0.2.2-rc.1".
                if v.chars().next().is_some_and(|c| c.is_ascii_digit())
                    && v.chars().all(|c| c.is_ascii_alphanumeric() || ".-+".contains(c))
                {
                    return Some(v);
                }
                return None;
            }
            Ok(None) => {
                if start.elapsed() > VERSION_PROBE_TIMEOUT {
                    child.kill().ok();
                    return None;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(_) => return None,
        }
    }
}

/// Poll a spawned child to completion with a timeout; stdio must be nulled
/// or drained by the caller (piped output can fill the pipe buffer and
/// deadlock the poll loop).
fn wait_child(child: &mut std::process::Child, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    Ok(())
                } else {
                    Err(format!("exit code {:?}", status.code()))
                };
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    child.kill().ok();
                    return Err("timed out".into());
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => return Err(format!("wait failed: {}", e)),
        }
    }
}

/// Run `dsh plugin --profile <profile> <op> <pkg-spec>` with a timeout.
/// stdio is nulled rather than piped: pnpm output can fill the pipe buffer
/// and deadlock a try_wait poll loop.
fn run_plugin_op(
    runner: DshRunner,
    op: &str,
    profile: &str,
    pkg_spec: &str,
) -> Result<(), String> {
    let args = ["plugin", "--profile", profile, op, pkg_spec];
    let mut child = dsh_command(runner, &args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to spawn dsh: {}", e))?;

    let timeout = runner.timeout();
    wait_child(&mut child, timeout)
}

const PNPM_INSTALL_TIMEOUT: Duration = Duration::from_secs(300);

/// `dsh plugin` forwards to pnpm, so it is a hard dependency — but any
/// machine that can run dsh has npm, so install pnpm globally instead of
/// failing. Returns Ok(true) when pnpm was auto-installed.
fn ensure_pnpm() -> Result<bool, String> {
    if on_path("pnpm") {
        return Ok(false);
    }
    if !on_path("npm") {
        return Err(
            "pnpm not found on PATH and npm unavailable — install pnpm manually (`npm i -g pnpm`)"
                .into(),
        );
    }

    // npm is an npm.cmd shim on Windows; go through cmd.exe like dsh_command.
    #[cfg(windows)]
    let mut cmd = {
        let mut c = Command::new("cmd");
        c.arg("/c").args(["npm", "i", "-g", "pnpm"]);
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        c.creation_flags(CREATE_NO_WINDOW);
        c
    };
    #[cfg(not(windows))]
    let mut cmd = {
        let mut c = Command::new("npm");
        c.args(["i", "-g", "pnpm"]);
        c
    };

    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to spawn npm: {}", e))?;
    wait_child(&mut child, PNPM_INSTALL_TIMEOUT)
        .map_err(|e| format!("auto-install of pnpm failed ({}) — run `npm i -g pnpm` manually", e))?;

    if on_path("pnpm") {
        Ok(true)
    } else {
        Err("pnpm still not on PATH after `npm i -g pnpm` — install it manually".into())
    }
}

/// Package specs to try for `plugin add`, most specific first. The bridge is
/// published in lockstep with dsh, but its `latest` dist-tag can lag behind
/// the runtime, so an exact version match is attempted before `@next` and
/// the default tag. dsh's own compatibility check rejects mismatches, moving
/// us to the next candidate.
fn add_candidates(pkg: &str, dsh_version: Option<&str>) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(ver) = dsh_version {
        v.push(format!("{}@{}", pkg, ver));
    }
    v.push(format!("{}@next", pkg));
    v.push(pkg.to_string());
    v
}

/// `plugin add` a package, walking the version candidates.
fn add_package(
    runner: DshRunner,
    profile: &str,
    pkg: &str,
    dsh_version: Option<&str>,
) -> Result<(), String> {
    add_candidates(pkg, dsh_version)
        .iter()
        .find_map(|spec| match run_plugin_op(runner, "add", profile, spec) {
            Ok(()) => Some(Ok(())),
            Err(_) => None,
        })
        .unwrap_or_else(|| Err(format!("all version candidates rejected for {}", pkg)))
}

/// Apply `dsh plugin <op>` for the bridge package to every existing profile.
/// Blocking (spawns subprocesses) — call from spawn_blocking. `progress`
/// receives short human-readable step updates for the UI.
fn bridge_op(home_dir: &Path, op: &str, progress: &dyn Fn(&str)) -> BridgeInstallOutcome {
    let mut outcome = BridgeInstallOutcome {
        runner: detect_runner(),
        ..Default::default()
    };
    let Some(runner) = outcome.runner else {
        return outcome;
    };

    // `dsh plugin` forwards to pnpm in the profile directory; install it via
    // npm when missing rather than failing.
    progress("checking pnpm…");
    match ensure_pnpm() {
        Ok(auto_installed) => {
            outcome.pnpm_auto_installed = auto_installed;
            if auto_installed {
                progress("pnpm installed via npm");
            }
        }
        Err(e) => {
            outcome.failed.push(("*".to_string(), e));
            return outcome;
        }
    }

    if op == "add" {
        progress("detecting dsh version…");
        outcome.dsh_version = probe_dsh_version(runner);
    }

    outcome.profiles = list_profiles(home_dir);
    for profile in outcome.profiles.clone() {
        if op == "add" {
            progress(&format!("installing bridge into profile {}…", profile));
        } else {
            progress(&format!("removing bridge from profile {}…", profile));
        }
        let result = if op == "add" {
            let ver = outcome.dsh_version.as_deref();
            add_package(runner, &profile, DSH_BRIDGE_PLUGIN, ver)
                .and_then(|()| add_package(runner, &profile, DSH_HOOK_PROTOCOL, ver))
        } else {
            let r = run_plugin_op(runner, op, &profile, DSH_BRIDGE_PLUGIN);
            // Best-effort: the protocol package may not be present (e.g. the
            // bridge was installed by an older deepNotifier version).
            let _ = run_plugin_op(runner, op, &profile, DSH_HOOK_PROTOCOL);
            r
        };
        match result {
            Ok(()) => outcome.installed.push(profile),
            Err(e) => {
                progress(&format!("profile {} failed: {}", profile, e));
                outcome.failed.push((profile, e));
            }
        }
    }
    outcome
}

/// Install the hook bridge package into every existing dsh profile.
pub fn ensure_bridge_installed(home_dir: &Path, progress: &dyn Fn(&str)) -> BridgeInstallOutcome {
    bridge_op(home_dir, "add", progress)
}

/// Remove the hook bridge package from every existing dsh profile.
pub fn uninstall_bridge(home_dir: &Path, progress: &dyn Fn(&str)) -> BridgeInstallOutcome {
    bridge_op(home_dir, "remove", progress)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_profiles_only_counts_initialised_dirs() {
        let dir = std::env::temp_dir().join(format!("dn-dsh-profiles-{}", std::process::id()));
        let profiles = dir.join("profiles");
        fs_create(&profiles.join("web"), "package.json");
        fs_create(&profiles.join("headless"), "cordis.yml");
        std::fs::create_dir_all(profiles.join("empty-dir")).unwrap();
        std::fs::write(profiles.join("stray-file.txt"), "x").unwrap();

        assert_eq!(list_profiles_in(&profiles), ["headless", "web"]);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn add_candidates_prefers_exact_version() {
        let c = add_candidates(DSH_BRIDGE_PLUGIN, Some("0.2.2-rc.1"));
        assert_eq!(
            c,
            [
                "@deepseek-ai/dsh-hooks-claude-code@0.2.2-rc.1",
                "@deepseek-ai/dsh-hooks-claude-code@next",
                "@deepseek-ai/dsh-hooks-claude-code",
            ]
        );
        let c = add_candidates(DSH_HOOK_PROTOCOL, None);
        assert_eq!(c.len(), 2);
        assert_eq!(c[0], "@deepseek-ai/dsh-hook-protocol@next");
    }

    fn fs_create(dir: &Path, file: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join(file), "{}").unwrap();
    }
}
