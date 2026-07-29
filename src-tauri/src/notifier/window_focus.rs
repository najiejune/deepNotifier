use std::collections::HashMap;
use std::mem;
use std::path::Path;

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
    PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AllowSetForegroundWindow, EnumWindows, GetAncestor, GetForegroundWindow, GetWindow,
    GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsWindowVisible, SetForegroundWindow, ShowWindow, ASFW_ANY,
    GA_ROOT, GW_OWNER, GWL_EXSTYLE, SW_RESTORE, WS_EX_TOOLWINDOW,
};
use windows::core::{BOOL, PWSTR};

pub use windows::Win32::System::Console::{
    AttachConsole, FreeConsole, GetConsoleWindow,
};

use ntapi::ntpebteb::PEB;
use ntapi::ntpsapi::{
    NtQueryInformationProcess, ProcessBasicInformation, PROCESS_BASIC_INFORMATION,
};
use ntapi::ntrtl::RTL_USER_PROCESS_PARAMETERS;

// ── Local UNICODE_STRING (ntapi's is private) ──────────────────────

#[repr(C)]
#[allow(non_snake_case)]
struct UnicodeStringLocal {
    Length: u16,
    MaximumLength: u16,
    Buffer: *mut u16,
}

// Offsets into RTL_USER_PROCESS_PARAMETERS for CurrentDirectory.DosPath.
// RTL_USER_PROCESS_PARAMETERS layout (simplified, x64):
//   +0x00: MaximumLength, Length (UNICODE_STRING)  - 4 bytes
//   +0x04: padding                                   - 4 bytes
//   +0x08: Buffer (*mut u16)                         - 8 bytes
//   ... (many fields)
//   CurrentDirectory is at a specific offset. We read the struct directly
//   from ntapi and access its fields, but for UNICODE_STRING we need to
//   manually read since ntapi's UNICODE_STRING is private.
//
// The RTL_USER_PROCESS_PARAMETERS from ntapi exposes:
//   - Environment: *mut u16
//   - EnvironmentSize: usize
//   - CurrentDirectory: ??? (might be UNICODE_STRING)
//
// Let's check what ntapi actually exposes...

/// Known IDE / terminal host process names.
const HOST_PROCESSES: &[&str] = &[
    "Code.exe", "Code - Insiders.exe", "VSCodium.exe", "Cursor.exe",
    "windsurf.exe", "Trae.exe",
    "goland64.exe", "idea64.exe", "pycharm64.exe", "webstorm64.exe",
    "clion64.exe", "rider64.exe", "phpstorm64.exe", "rubymine64.exe",
    "datagrip64.exe", "studio64.exe", "fleet.exe",
    "WindowsTerminal.exe", "wt.exe",
    "powershell.exe", "pwsh.exe", "cmd.exe",
    "alacritty.exe", "wezterm-gui.exe", "mintty.exe",
];

// ── Process snapshot ──────────────────────────────────────────────

#[derive(Debug, Clone)]
struct ProcessInfo {
    pid: u32,
    parent_pid: u32,
    name: String,
}

fn snapshot_processes() -> Option<Vec<ProcessInfo>> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).ok()?;
        let mut entry: PROCESSENTRY32 = mem::zeroed();
        entry.dwSize = mem::size_of::<PROCESSENTRY32>() as u32;
        let mut result = Vec::new();
        if Process32First(snapshot, &mut entry).is_ok() {
            loop {
                // szExeFile is [i8; 260] in windows 0.62, cast to u8 then u16
                let name_bytes: &[i8] = &entry.szExeFile;
                let end = name_bytes.iter().position(|&c| c == 0).unwrap_or(name_bytes.len());
                let name: String = name_bytes[..end]
                    .iter()
                    .map(|&c| c as u8 as char)
                    .collect();
                result.push(ProcessInfo {
                    pid: entry.th32ProcessID,
                    parent_pid: entry.th32ParentProcessID,
                    name,
                });
                if Process32Next(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
        Some(result)
    }
}

fn get_process_path(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 260];
        let mut size = buf.len() as u32;
        let r = QueryFullProcessImageNameW(
            h,
            PROCESS_NAME_FORMAT(0),
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(h);
        if r.is_ok() {
            Some(String::from_utf16_lossy(&buf[..size as usize]))
        } else {
            None
        }
    }
}

fn is_host(name: &str) -> bool {
    HOST_PROCESSES.iter().any(|h| h.eq_ignore_ascii_case(name))
}

// ── Strategy 3: find host + all same-name process windows ─────────

/// True if the process name is a console subsystem host (its window is owned by conhost.exe).
fn is_console_host(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "powershell.exe" | "pwsh.exe" | "cmd.exe"
    )
}

/// Get the conhost.exe PID that owns the console window for the given process.
/// Uses NtQueryInformationProcess(ProcessConsoleHostProcess).
fn get_console_host_pid(pid: u32) -> Option<u32> {
    use ntapi::ntpsapi::ProcessConsoleHostProcess;
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut value: usize = 0;
        let status = NtQueryInformationProcess(
            h.0 as _,
            ProcessConsoleHostProcess,
            &mut value as *mut _ as *mut _,
            std::mem::size_of::<usize>() as u32,
            std::ptr::null_mut(),
        );
        let _ = CloseHandle(h);
        if status < 0 || value == 0 {
            return None;
        }
        Some((value & 0xFFFF_FFFF) as u32)
    }
}

/// Walk parent chain to find the first known IDE/terminal host process.
fn find_host_process(map: &HashMap<u32, ProcessInfo>, start_pid: u32) -> Option<ProcessInfo> {
    let mut current = start_pid;
    let mut visited = std::collections::HashSet::new();
    while visited.insert(current) {
        let p = map.get(&current)?;
        if is_host(&p.name) {
            return Some(p.clone());
        }
        if p.parent_pid == 0 || p.parent_pid == p.pid {
            return None;
        }
        current = p.parent_pid;
    }
    None
}

/// Get all PIDs whose exe name matches the given process name.
fn find_all_same_name_pids(map: &HashMap<u32, ProcessInfo>, name: &str) -> Vec<u32> {
    map.values()
        .filter(|p| p.name.eq_ignore_ascii_case(name))
        .map(|p| p.pid)
        .collect()
}

// ── Window enumeration ────────────────────────────────────────────

#[derive(Debug, Clone)]
struct WindowInfo {
    hwnd: HWND,
    pid: u32,
    title: String,
    _rect: RECT,
}

struct EnumCtx {
    target_pid: u32,
    windows: Vec<WindowInfo>,
}

/// Enumerate ALL visible top-level windows belonging to `pid`.
fn enum_windows_for_pid(pid: u32) -> Vec<WindowInfo> {
    let mut ctx = EnumCtx {
        target_pid: pid,
        windows: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(
            Some(enum_window_proc),
            LPARAM((&mut ctx) as *mut EnumCtx as isize),
        );
    }
    ctx.windows
}

unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam.0 as *mut EnumCtx);
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid != ctx.target_pid {
        return BOOL::from(true);
    }
    // Skip owned windows (child dialogs, popups)
    let owner = GetWindow(hwnd, GW_OWNER).unwrap_or(HWND(std::ptr::null_mut()));
    if !owner.0.is_null() {
        return BOOL::from(true);
    }
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL::from(true);
    }
    // Skip tool windows
    let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
    if (ex & WS_EX_TOOLWINDOW.0) != 0 {
        return BOOL::from(true);
    }
    // Skip cloaked windows
    let mut cloaked: u32 = 0;
    let _ = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked as *mut _ as *mut _,
        mem::size_of::<u32>() as u32,
    );
    if cloaked != 0 {
        return BOOL::from(true);
    }
    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_err() {
        return BOOL::from(true);
    }
    let len = GetWindowTextLengthW(hwnd);
    let title = if len > 0 {
        let mut buf = vec![0u16; (len + 1) as usize];
        let n = GetWindowTextW(hwnd, &mut buf);
        String::from_utf16_lossy(&buf[..n as usize])
    } else {
        String::new()
    };
    ctx.windows.push(WindowInfo { hwnd, pid, title, _rect: rect });
    BOOL::from(true)
}

/// Pick best window: prefer one with a title, otherwise the first.
fn pick_best_window(ws: &[WindowInfo]) -> Option<&WindowInfo> {
    ws.iter().find(|w| !w.title.is_empty()).or_else(|| ws.first())
}

// ── Strategy 4: CWD → window title matching ───────────────────────

/// Read another process's CWD by walking PEB → ProcessParameters → CurrentDirectory.
///
/// Since ntapi's UNICODE_STRING is private, we manually read the DosPath from
/// the CurrentDirectory offset within RTL_USER_PROCESS_PARAMETERS.
fn read_process_cwd(pid: u32) -> Option<String> {
    unsafe {
        let h = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            pid,
        )
        .ok()?;

        let mut pbi: PROCESS_BASIC_INFORMATION = mem::zeroed();
        let status = NtQueryInformationProcess(
            h.0 as _,
            ProcessBasicInformation,
            &mut pbi as *mut _ as *mut _,
            mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
            std::ptr::null_mut(),
        );
        if status < 0 || pbi.PebBaseAddress.is_null() {
            let _ = CloseHandle(h);
            return None;
        }

        // Read PEB
        let mut peb: PEB = mem::zeroed();
        if ReadProcessMemory(
            h,
            pbi.PebBaseAddress as _,
            &mut peb as *mut _ as _,
            mem::size_of::<PEB>(),
            None,
        )
        .is_err()
        {
            let _ = CloseHandle(h);
            return None;
        }

        // Read ProcessParameters
        let mut params: RTL_USER_PROCESS_PARAMETERS = mem::zeroed();
        if ReadProcessMemory(
            h,
            peb.ProcessParameters as _,
            &mut params as *mut _ as _,
            mem::size_of::<RTL_USER_PROCESS_PARAMETERS>(),
            None,
        )
        .is_err()
        {
            let _ = CloseHandle(h);
            return None;
        }

        // RTL_USER_PROCESS_PARAMETERS layout (x64, offset confirmed via windbg/dt):
        // The CurrentDirectory field contains a UNICODE_STRING (4 bytes header + pointer).
        // ntapi's RTL_USER_PROCESS_PARAMETERS has various fields before CurrentDirectory.
        //
        // Simpler approach: use the Environment field which IS public in ntapi's struct,
        // or use a fixed offset approach.
        //
        // Actually, let's try a different strategy: read the struct and then use pointer
        // arithmetic to get CurrentDirectory.DosPath.
        //
        // On x64, RTL_USER_PROCESS_PARAMETERS offsets (verified for Windows 10/11):
        // +0x038 CurrentDirectory : UNICODE_STRING  (offset may vary between builds!)
        //
        // Safer: define just the fields we need up to CurrentDirectory.
        let cwd = read_current_directory_from_params(h, &params);
        let _ = CloseHandle(h);
        cwd
    }
}

/// Read CurrentDirectory.DosPath from RTL_USER_PROCESS_PARAMETERS using
/// field access on the ntapi struct. The struct has public fields for
/// everything except the UNICODE_STRING internals. We can access
/// CurrentDirectory as a field and then read its Buffer manually.
unsafe fn read_current_directory_from_params(
    h: HANDLE,
    params: &RTL_USER_PROCESS_PARAMETERS,
) -> Option<String> {
    // ntapi's RTL_USER_PROCESS_PARAMETERS layout:
    // The CurrentDirectory field IS accessible. The issue is that
    // its type is UNICODE_STRING which has private fields in ntapi 0.4.
    //
    // We work around this by reading the raw bytes of CurrentDirectory
    // from the process memory at the field's offset, interpreting them
    // as our local UnicodeStringLocal.

    // Offset of CurrentDirectory within RTL_USER_PROCESS_PARAMETERS on x64.
    // This is the standard offset for Windows 10/11 64-bit.
    // RTL_USER_PROCESS_PARAMETERS:
    //   +0x000 MaximumLength   : UInt4B
    //   +0x004 Length          : UInt4B
    //   +0x008 Flags           : UInt4B
    //   +0x00c DebugFlags      : UInt4B
    //   +0x010 ConsoleHandle   : Ptr64 Void
    //   +0x018 ConsoleFlags    : UInt4B
    //   +0x020 StandardInput   : Ptr64 Void
    //   +0x028 StandardOutput  : Ptr64 Void
    //   +0x030 StandardError   : Ptr64 Void
    //   +0x038 CurrentDirectory : UNICODE_STRING

    // Actually, a much simpler approach: use ntapi's public API to access
    // the CurrentDirectory field. Let me check what fields are public...

    // Since we can't access UNICODE_STRING fields directly, let's use
    // pointer arithmetic: params_ptr + 0x38 = address of CurrentDirectory
    let params_ptr = params as *const RTL_USER_PROCESS_PARAMETERS as usize;
    let cwd_offset = 0x38; // CurrentDirectory offset in x64 RTL_USER_PROCESS_PARAMETERS
    let cwd_addr = (params_ptr + cwd_offset) as *const UnicodeStringLocal;

    // Read the UNICODE_STRING header from our own memory (we already have params locally)
    let cwd_us: UnicodeStringLocal = std::ptr::read(cwd_addr);

    if cwd_us.Length == 0 || cwd_us.Buffer.is_null() {
        return None;
    }

    let len = cwd_us.Length as usize / 2;
    let mut buf = vec![0u16; len];
    if ReadProcessMemory(
        h,
        cwd_us.Buffer as _,
        buf.as_mut_ptr() as _,
        cwd_us.Length as usize,
        None,
    )
    .is_ok()
    {
        Some(
            String::from_utf16_lossy(&buf)
                .trim_end_matches('\\')
                .to_string(),
        )
    } else {
        None
    }
}

/// Score windows by how many CWD path segments appear in the window title.
/// Each matching segment scores its (index + 1), so deeper path components
/// (closer to the project name) carry more weight.
fn match_window_by_cwd<'a>(
    windows: &'a [WindowInfo],
    cwd: Option<&str>,
) -> Option<(&'a WindowInfo, usize)> {
    let cwd = cwd?;
    let path = Path::new(cwd);
    let segments: Vec<String> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .filter(|s| s.len() > 1 && !s.ends_with(':'))
        .map(|s| s.to_lowercase())
        .collect();
    if segments.is_empty() {
        return None;
    }

    let mut best: Option<(&WindowInfo, usize)> = None;
    for w in windows {
        let t = w.title.to_lowercase();
        if t.is_empty() {
            continue;
        }
        let mut score = 0usize;
        for (i, seg) in segments.iter().enumerate() {
            if t.contains(seg) {
                score += i + 1; // deeper segments weight more
            }
        }
        if score > 0 {
            match best {
                None => best = Some((w, score)),
                Some((_, s)) if score > s => best = Some((w, score)),
                _ => {}
            }
        }
    }
    best
}

// ── Public API (preserved signatures) ─────────────────────────────

/// Capture the PID of the current foreground window.
pub fn capture_foreground_pid() -> Option<u32> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 { None } else { Some(pid) }
    }
}

/// Try to get the console window for a process using AttachConsole + GetConsoleWindow.
fn find_console_window_for_pid(pid: u32) -> Option<HWND> {
    unsafe {
        let _ = FreeConsole();
        if AttachConsole(pid).is_err() {
            return None;
        }
        let hwnd = GetConsoleWindow();
        let _ = FreeConsole();
        if hwnd.0.is_null() {
            return None;
        }
        let root = GetAncestor(hwnd, GA_ROOT);
        let result = if root.0.is_null() { hwnd } else { root };
        tracing::info!(pid, ?hwnd, ?result, "find_console_window_for_pid: found via AttachConsole");
        Some(result)
    }
}

/// Resolve a CLI process PID to the terminal/IDE PID that owns the visible window.
///
/// Resolution order:
/// 1. `AttachConsole` + `GetConsoleWindow` — exact terminal window
/// 2. Original PID already has a visible window
/// 3. Walk parent chain → find host → enumerate all same-name process windows + CWD match
/// 4. Return original PID as fallback
pub fn resolve_terminal_pid(pid: u32) -> u32 {
    // 1. AttachConsole
    if let Some(hwnd) = find_console_window_for_pid(pid) {
        let mut wpid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut wpid)); }
        if wpid != 0 {
            tracing::info!(pid, terminal_pid = wpid, "resolve_terminal_pid: found via AttachConsole");
            return wpid;
        }
    }

    // 2. Original PID has a window?
    if !enum_windows_for_pid(pid).is_empty() {
        tracing::info!(pid, "resolve_terminal_pid: original PID has visible window");
        return pid;
    }

    // 3. Strategy 3+4: parent chain → host → same-name siblings + CWD match
    if let Some(proc_map) = snapshot_processes()
        .map(|ps| ps.into_iter().map(|p| (p.pid, p)).collect::<HashMap<_, _>>())
    {
        if let Some(host) = find_host_process(&proc_map, pid) {
            tracing::info!(pid, host_pid = host.pid, host_name = %host.name, "resolve_terminal_pid: found host");

            let all_host_pids = find_all_same_name_pids(&proc_map, &host.name);
            let mut all_windows: Vec<WindowInfo> = Vec::new();
            for hpid in &all_host_pids {
                all_windows.extend(enum_windows_for_pid(*hpid));
            }

            if all_windows.is_empty() {
                // Console host (cmd/pwsh) has no visible window — find conhost, then
                // continue walking up to find the real terminal (WindowsTerminal/Code).
                if is_console_host(&host.name) {
                    // Try conhost first
                    if let Some(conhost_pid) = get_console_host_pid(host.pid) {
                        let conhost_windows = enum_windows_for_pid(conhost_pid);
                        if let Some(w) = pick_best_window(&conhost_windows) {
                            tracing::info!(pid, window_pid = w.pid, ?w.hwnd, "resolve_terminal_pid: found via console host");
                            return w.pid;
                        }
                    }
                    // Try AttachConsole on the host
                    if let Some(hwnd) = find_console_window_for_pid(host.pid) {
                        let mut wpid: u32 = 0;
                        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut wpid)); }
                        if wpid != 0 {
                            tracing::info!(pid, window_pid = wpid, "resolve_terminal_pid: found via AttachConsole on host");
                            return wpid;
                        }
                    }
                    // Continue walking up from host's parent to find a real terminal
                    if let Some(upper_host) = find_host_process(&proc_map, host.parent_pid) {
                        tracing::info!(pid, upper_pid = upper_host.pid, upper_name = %upper_host.name, "resolve_terminal_pid: walking up past console host");
                        let upper_pids = find_all_same_name_pids(&proc_map, &upper_host.name);
                        let mut upper_windows: Vec<WindowInfo> = Vec::new();
                        for upid in &upper_pids {
                            upper_windows.extend(enum_windows_for_pid(*upid));
                        }
                        if !upper_windows.is_empty() {
                            let cwd = read_process_cwd(pid);
                            if let Some((best, score)) = match_window_by_cwd(&upper_windows, cwd.as_deref()) {
                                tracing::info!(pid, window_pid = best.pid, score, title = %best.title, "resolve_terminal_pid: upper host CWD match");
                                return best.pid;
                            }
                            if let Some(w) = pick_best_window(&upper_windows) {
                                tracing::info!(pid, window_pid = w.pid, "resolve_terminal_pid: upper host fallback");
                                return w.pid;
                            }
                        }
                    }
                }
                tracing::warn!(pid, "resolve_terminal_pid: host has no visible windows");
            } else if all_windows.len() == 1 {
                tracing::info!(pid, window_pid = all_windows[0].pid, "resolve_terminal_pid: single host window");
                return all_windows[0].pid;
            } else {
                // Strategy 4: CWD match
                let cwd = read_process_cwd(pid);
                tracing::info!(pid, ?cwd, "resolve_terminal_pid: multiple host windows, trying CWD match");
                if let Some((best, score)) = match_window_by_cwd(&all_windows, cwd.as_deref()) {
                    tracing::info!(pid, window_pid = best.pid, score, title = %best.title, "resolve_terminal_pid: CWD match");
                    return best.pid;
                }
                // Fallback
                if let Some(w) = pick_best_window(&all_windows) {
                    tracing::info!(pid, window_pid = w.pid, "resolve_terminal_pid: fallback best window");
                    return w.pid;
                }
            }
        }
    }

    // 4. Fallback
    tracing::warn!(pid, "resolve_terminal_pid: could not resolve, returning original");
    pid
}

/// Diagnostic version: resolve a CLI PID and return a detailed step-by-step report.
/// Used by the test_resolve binary to verify window resolution correctness.
pub fn debug_resolve(pid: u32) -> String {
    let mut report = String::new();
    use std::fmt::Write;

    let _ = writeln!(report, "═══ Debug resolve PID: {} ═══", pid);

    // Basic process info
    let proc_map: HashMap<u32, ProcessInfo> = match snapshot_processes() {
        Some(ps) => {
            let m: HashMap<u32, ProcessInfo> = ps.into_iter().map(|p| (p.pid, p)).collect();
            if let Some(p) = m.get(&pid) {
                let _ = writeln!(report, "  Process: {}  parent_pid={}",
                    p.name, p.parent_pid);
                let exe = get_process_path(pid);
                let _ = writeln!(report, "  ExePath: {:?}", exe);
            } else {
                let _ = writeln!(report, "  Process: NOT FOUND in snapshot (already exited?)");
            }
            m
        }
        None => {
            let _ = writeln!(report, "  ERROR: Failed to take process snapshot");
            return report;
        }
    };

    // Step 1: AttachConsole
    let _ = writeln!(report, "--- Step 1: AttachConsole ---");
    if let Some(hwnd) = find_console_window_for_pid(pid) {
        let mut wpid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut wpid)); }
        let _ = writeln!(report, "  Found: hwnd={:?} pid={}", hwnd, wpid);
        let _ = writeln!(report, "  RESULT: {}", wpid);
        return report;
    }
    let _ = writeln!(report, "  Not found via AttachConsole");

    // Step 2: Original PID windows
    let _ = writeln!(report, "--- Step 2: Original PID windows ---");
    {
        let ws = enum_windows_for_pid(pid);
        if !ws.is_empty() {
            for w in &ws {
                let _ = writeln!(report, "  Window: hwnd={:?} pid={} title='{}'",
                    w.hwnd, w.pid, w.title);
            }
            let _ = writeln!(report, "  RESULT: {} (original PID has visible windows)", pid);
            return report;
        }
        let _ = writeln!(report, "  No visible windows for original PID");
    }

    // Step 3: Parent chain → host
    let _ = writeln!(report, "--- Step 3: Parent chain ---");

    // Print parent chain
    {
        let mut current = pid;
        let mut level = 0;
        let mut visited = std::collections::HashSet::new();
        while visited.insert(current) && level < 20 {
            if let Some(p) = proc_map.get(&current) {
                let tag = if is_host(&p.name) { " [HOST]" } else { "" };
                let _ = writeln!(report, "  {} -> {} (pid={}){}",
                    level, p.name, p.pid, tag);
                if p.parent_pid == 0 || p.parent_pid == p.pid { break; }
                current = p.parent_pid;
                level += 1;
            } else {
                let _ = writeln!(report, "  {} -> (unknown ppid) - chain broken", level + 1);
                break;
            }
        }
    }

    let host = match find_host_process(&proc_map, pid) {
        Some(h) => {
            let _ = writeln!(report, "  Found host: {} (pid={})", h.name, h.pid);
            h
        }
        None => {
            let _ = writeln!(report, "  No known host found in parent chain");
            let _ = writeln!(report, "  RESULT: {} (fallback to original)", pid);
            return report;
        }
    };

    // Enumerate all same-name host windows
    let _ = writeln!(report, "--- Step 3a: Host windows (all same-name PIDs) ---");
    let all_host_pids = find_all_same_name_pids(&proc_map, &host.name);
    let _ = writeln!(report, "  Host PIDs: {:?}", all_host_pids);
    let mut all_windows: Vec<WindowInfo> = Vec::new();
    for hpid in &all_host_pids {
        let ws = enum_windows_for_pid(*hpid);
        for w in &ws {
            let _ = writeln!(report, "    hwnd={:?} pid={} title='{}'",
                w.hwnd, w.pid, w.title);
        }
        all_windows.extend(ws);
    }
    let _ = writeln!(report, "  Total visible windows: {}", all_windows.len());

    if all_windows.is_empty() {
        let _ = writeln!(report, "--- Step 3b: Console host fallback ---");
        if is_console_host(&host.name) {
            let _ = writeln!(report, "  Host is console host ({}), trying conhost...", host.name);
            if let Some(conhost_pid) = get_console_host_pid(host.pid) {
                let _ = writeln!(report, "  conhost PID: {}", conhost_pid);
                let conhost_windows = enum_windows_for_pid(conhost_pid);
                for w in &conhost_windows {
                    let _ = writeln!(report, "    hwnd={:?} pid={} title='{}'",
                        w.hwnd, w.pid, w.title);
                }
                if let Some(w) = pick_best_window(&conhost_windows) {
                    let _ = writeln!(report, "  RESULT: {} (via conhost)", w.pid);
                    return report;
                }
            }
            if let Some(hwnd) = find_console_window_for_pid(host.pid) {
                let mut wpid: u32 = 0;
                unsafe { GetWindowThreadProcessId(hwnd, Some(&mut wpid)); }
                let _ = writeln!(report, "  RESULT: {} (via AttachConsole on host)", wpid);
                return report;
            }
            // Walk up
            let _ = writeln!(report, "  Walking up past console host...");
            if let Some(upper) = find_host_process(&proc_map, host.parent_pid) {
                let _ = writeln!(report, "  Upper host: {} (pid={})", upper.name, upper.pid);
                let upper_pids = find_all_same_name_pids(&proc_map, &upper.name);
                for upid in &upper_pids {
                    let ws = enum_windows_for_pid(*upid);
                    for w in &ws {
                        let _ = writeln!(report, "    hwnd={:?} pid={} title='{}'",
                            w.hwnd, w.pid, w.title);
                    }
                    all_windows.extend(ws);
                }
            }
        }
    }

    if all_windows.is_empty() {
        let _ = writeln!(report, "  RESULT: {} (no visible windows, fallback)", pid);
        return report;
    }

    // Single window
    if all_windows.len() == 1 {
        let _ = writeln!(report, "  RESULT: {} (single host window)", all_windows[0].pid);
        return report;
    }

    // Step 4: CWD match
    let _ = writeln!(report, "--- Step 4: CWD match ---");
    let cwd = read_process_cwd(pid);
    let _ = writeln!(report, "  CWD: {:?}", cwd);

    if let Some((best, score)) = match_window_by_cwd(&all_windows, cwd.as_deref()) {
        let _ = writeln!(report, "  Best match: pid={} title='{}' score={}",
            best.pid, best.title, score);
        let _ = writeln!(report, "  RESULT: {} (CWD match)", best.pid);
        return report;
    }

    // Fallback
    let w = pick_best_window(&all_windows);
    let _ = writeln!(report, "  Fallback: {:?}", w.map(|w| (w.pid, &w.title)));
    let _ = writeln!(report, "  RESULT: {} (fallback best window)",
        w.map_or(pid, |w| w.pid));
    report
}

/// Bring the window of a given process to the foreground.
pub fn bring_pid_to_front(pid: u32) {
    tracing::info!(pid, "bring_pid_to_front: spawning focus thread");
    std::thread::spawn(move || {
        focus_windows(pid);
    });
}

fn focus_windows(pid: u32) {
    tracing::info!(pid, "focus_windows: looking for visible window");
    let hwnd = find_visible_window(pid);

    let Some(hwnd) = hwnd else {
        tracing::warn!(pid, "focus_windows: No visible top-level window found");
        return;
    };

    tracing::info!(pid, ?hwnd, "focus_windows: found window, calling SetForegroundWindow");
    unsafe {
        let _ = AllowSetForegroundWindow(ASFW_ANY);
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetForegroundWindow(hwnd);
    }
}

/// Walk PID → parent chain, then same-name host processes, looking for a visible top-level window.
fn find_visible_window(pid: u32) -> Option<HWND> {
    // 1. Check PID directly
    let direct = enum_windows_for_pid(pid);
    if let Some(w) = pick_best_window(&direct) {
        tracing::info!(pid, ?w.hwnd, "find_visible_window: found directly");
        return Some(w.hwnd);
    }

    // 2. Strategy 3+4: parent chain → host → same-name siblings + CWD match
    let proc_map: HashMap<u32, ProcessInfo> = snapshot_processes()?
        .into_iter()
        .map(|p| (p.pid, p))
        .collect();

    let host = find_host_process(&proc_map, pid)?;
    tracing::info!(pid, host_pid = host.pid, host_name = %host.name, "find_visible_window: found host");

    let all_host_pids = find_all_same_name_pids(&proc_map, &host.name);
    let mut all_windows: Vec<WindowInfo> = Vec::new();
    for hpid in &all_host_pids {
        all_windows.extend(enum_windows_for_pid(*hpid));
    }

    if all_windows.is_empty() {
        // Console host fallback
        if is_console_host(&host.name) {
            if let Some(conhost_pid) = get_console_host_pid(host.pid) {
                let conhost_windows = enum_windows_for_pid(conhost_pid);
                if let Some(w) = pick_best_window(&conhost_windows) {
                    return Some(w.hwnd);
                }
            }
            if let Some(hwnd) = find_console_window_for_pid(host.pid) {
                return Some(hwnd);
            }
            // Walk up past console host
            if let Some(upper_host) = find_host_process(&proc_map, host.parent_pid) {
                let upper_pids = find_all_same_name_pids(&proc_map, &upper_host.name);
                let mut upper_windows: Vec<WindowInfo> = Vec::new();
                for upid in &upper_pids {
                    upper_windows.extend(enum_windows_for_pid(*upid));
                }
                if !upper_windows.is_empty() {
                    let cwd = read_process_cwd(pid);
                    if let Some((best, _)) = match_window_by_cwd(&upper_windows, cwd.as_deref()) {
                        return Some(best.hwnd);
                    }
                    if let Some(w) = pick_best_window(&upper_windows) {
                        return Some(w.hwnd);
                    }
                }
            }
        }
        tracing::warn!(pid, "find_visible_window: host has no visible windows");
        return None;
    }

    if all_windows.len() == 1 {
        return Some(all_windows[0].hwnd);
    }

    // CWD match
    let cwd = read_process_cwd(pid);
    if let Some((best, _score)) = match_window_by_cwd(&all_windows, cwd.as_deref()) {
        tracing::info!(pid, ?best.hwnd, _score, title = %best.title, "find_visible_window: CWD match");
        return Some(best.hwnd);
    }

    pick_best_window(&all_windows).map(|w| w.hwnd)
}

// ── Diagnostic API ────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct FocusDiagnostic {
    pub pid: u32,
    pub direct_window_found: bool,
    pub parent_chain: Vec<u32>,
    pub window_found_via_parent: Option<u32>,
    pub success: bool,
}

pub fn diagnose_focus(pid: u32) -> FocusDiagnostic {
    let mut diag = FocusDiagnostic {
        pid,
        direct_window_found: false,
        parent_chain: Vec::new(),
        window_found_via_parent: None,
        success: false,
    };

    // Check direct PID
    let direct = enum_windows_for_pid(pid);
    if let Some(w) = pick_best_window(&direct) {
        diag.direct_window_found = true;
        diag.success = true;
        focus_window_hwnd(w.hwnd, pid);
        return diag;
    }

    // Walk parent chain via snapshot
    if let Some(proc_map) = snapshot_processes()
        .map(|ps| ps.into_iter().map(|p| (p.pid, p)).collect::<HashMap<_, _>>())
    {
        // Record parent chain
        let mut current = pid;
        let mut visited = std::collections::HashSet::new();
        while visited.insert(current) {
            if let Some(p) = proc_map.get(&current) {
                if p.parent_pid == 0 || p.parent_pid == p.pid {
                    break;
                }
                diag.parent_chain.push(p.parent_pid);
                current = p.parent_pid;
            } else {
                break;
            }
        }

        if let Some(host) = find_host_process(&proc_map, pid) {
            let all_host_pids = find_all_same_name_pids(&proc_map, &host.name);
            let mut all_windows: Vec<WindowInfo> = Vec::new();
            for hpid in &all_host_pids {
                all_windows.extend(enum_windows_for_pid(*hpid));
            }

            let found = if all_windows.len() == 1 {
                Some(all_windows[0].clone())
            } else {
                let cwd = read_process_cwd(pid);
                match_window_by_cwd(&all_windows, cwd.as_deref())
                    .map(|(w, _)| w.clone())
                    .or_else(|| pick_best_window(&all_windows).cloned())
            };

            if let Some(w) = found {
                diag.window_found_via_parent = Some(w.pid);
                diag.success = true;
                focus_window_hwnd(w.hwnd, w.pid);
                return diag;
            }
        }
    }

    diag
}

fn focus_window_hwnd(hwnd: HWND, pid: u32) {
    unsafe {
        let _ = AllowSetForegroundWindow(ASFW_ANY);
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetForegroundWindow(hwnd);
    }
    tracing::info!(pid, ?hwnd, "focus_window_hwnd: focus calls complete");
}
