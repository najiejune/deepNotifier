/// Generate a webhook POST command for the given event type.
///
/// Returns (unix_cmd, windows_cmd) — the injector picks based on platform.
/// Windows commands use Node.js (avoids PowerShell `$pid` variable consumption
/// by intermediate template engines). Unix commands use curl.
pub struct GeneratedCommand {
    pub unix: String,
    pub windows: String,
}

pub fn generate_webhook_command(
    port: u16,
    cli_id: &str,
    cli_name: &str,
    event_type: &str,
    approval_timeout_secs: u32,
) -> GeneratedCommand {
    match event_type {
        "stop" => generate_stop(port, cli_id, cli_name),
        "notification" => generate_notification(port, cli_id, cli_name),
        "pretooluse" => generate_pretooluse(port, cli_id, cli_name),
        "posttooluse" => generate_posttooluse(port, cli_id, approval_timeout_secs),
        _ => generate_stop(port, cli_id, cli_name),
    }
}

fn shell_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Escape a string for embedding in a JavaScript single-quoted string literal.
fn js_sq(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

// ---- stop ----

fn generate_stop(port: u16, cli_id: &str, cli_name: &str) -> GeneratedCommand {
    let url = format!("http://localhost:{}/hook/cli", port);
    let title = format!("{} finished", cli_name);
    let body = "Turn completed.";

    let unix = format!(
        r#"( curl -s --max-time 5 --connect-timeout 3 -X POST "{}" -H 'Content-Type: application/json' -d "$(printf '{{"title":"{}","body":"{}","cli_id":"{}","event_type":"stop","pid":%d}}' "$PPID")" 2>/dev/null || true ) &"#,
        url,
        shell_escape(&title),
        shell_escape(body),
        cli_id,
    );

    // Node.js: process.ppid → parent PID, JSON.stringify → safe JSON
    let node = format!(
        r#"node -e "try{{require('http').request({{hostname:'localhost',port:{},path:'/hook/cli',method:'POST',headers:{{'content-type':'application/json'}}}},r=>r.resume()).on('error',()=>{{}}).end(JSON.stringify({{title:'{}',body:'{}',cli_id:'{}',event_type:'stop',pid:process.ppid}}))}}catch(e){{}}""#,
        port,
        js_sq(&title),
        js_sq(body),
        cli_id,
    );

    GeneratedCommand { unix, windows: node }
}

// ---- notification ----

fn generate_notification(port: u16, cli_id: &str, cli_name: &str) -> GeneratedCommand {
    let url = format!("http://localhost:{}/hook/cli", port);
    let default_body = format!("{} notification", cli_name);
    let title = format!("{} notification", cli_name);

    let unix = format!(
        r#"( BODY=$(cat 2>/dev/null || echo "{}"); curl -s --max-time 5 --connect-timeout 3 -X POST "{}" -H 'Content-Type: application/json' -d "$(printf '{{"title":"{}","body":"%s","cli_id":"{}","event_type":"notification","pid":%d}}' "$(echo "$BODY" | sed 's/"/\\"/g')" "$PPID")" 2>/dev/null || true ) &"#,
        shell_escape(&default_body),
        url,
        shell_escape(&title),
        cli_id,
    );

    // Reads notification body from stdin, falls back to default if empty
    let node = format!(
        r#"node -e "try{{var s='';process.stdin.on('data',c=>s+=c);process.stdin.on('end',()=>{{var b=s.trim()||'{}';require('http').request({{hostname:'localhost',port:{},path:'/hook/cli',method:'POST',headers:{{'content-type':'application/json'}}}},r=>r.resume()).on('error',()=>{{}}).end(JSON.stringify({{title:'{}',body:b,cli_id:'{}',event_type:'notification',pid:process.ppid}}))}});process.stdin.resume()}}catch(e){{}}""#,
        js_sq(&default_body),
        port,
        js_sq(&title),
        cli_id,
    );

    GeneratedCommand { unix, windows: node }
}

// ---- pretooluse ----

fn generate_pretooluse(port: u16, cli_id: &str, cli_name: &str) -> GeneratedCommand {
    let url = format!("http://localhost:{}/hook/cli", port);

    let unix = format!(
        r#"( SID=$(uuidgen 2>/dev/null || echo $$-$RANDOM); mkdir -p "$HOME/.deepnotifier/approval" 2>/dev/null; printf '%s' "$SID" > "$HOME/.deepnotifier/approval/{}.$PPID"; curl -s --max-time 5 --connect-timeout 3 -X POST "{}" -H 'Content-Type: application/json' -d "$(printf '{{"title":"Tool use started","body":"Waiting for approval...","cli_id":"{}","cli_name":"{}","event_type":"pretooluse","session_id":"%s","pid":%d}}' "$SID" "$PPID")" 2>/dev/null || true ) &"#,
        cli_id, url, cli_id, shell_escape(cli_name),
    );

    // Marker file stores only session_id; timeout tracking is server-side
    let node = format!(
        r#"node -e "try{{var ppid=process.ppid;var sid=require('crypto').randomUUID();var p=require('os').homedir()+'/.deepnotifier/approval/';require('fs').mkdirSync(p,{{recursive:true}});require('fs').writeFileSync(p+'{}.'+ppid+'.txt',sid);require('http').request({{hostname:'localhost',port:{},path:'/hook/cli',method:'POST',headers:{{'content-type':'application/json'}}}},r=>r.resume()).on('error',()=>{{}}).end(JSON.stringify({{title:'Tool use started',body:'Waiting for approval...',cli_id:'{}',cli_name:'{}',event_type:'pretooluse',session_id:sid,pid:ppid}}))}}catch(e){{}}""#,
        cli_id, port, cli_id, js_sq(cli_name),
    );

    GeneratedCommand { unix, windows: node }
}

// ---- posttooluse ----

fn generate_posttooluse(port: u16, cli_id: &str, _timeout_secs: u32) -> GeneratedCommand {
    let url = format!("http://localhost:{}/hook/cli", port);

    // Read session_id from marker file, then always POST (server checks timeout)
    let unix = format!(
        r#"( MARKER="$HOME/.deepnotifier/approval/{}.$PPID"; SID=$(cat "$MARKER" 2>/dev/null || echo ""); rm -f "$MARKER" 2>/dev/null; if [ -n "$SID" ]; then curl -s --max-time 5 --connect-timeout 3 -X POST "{}" -H 'Content-Type: application/json' -d "$(printf '{{"title":"Tool use completed","body":"Approval done.","cli_id":"{}","event_type":"posttooluse","session_id":"%s","pid":%d}}' "$SID" "$PPID")" 2>/dev/null || true; fi ) &"#,
        cli_id, url, cli_id,
    );

    // Read session_id from marker file, POST to server (timeout is checked server-side)
    let node = format!(
        r#"node -e "try{{var ppid=process.ppid;var fs=require('fs');var f=require('os').homedir()+'/.deepnotifier/approval/'+'{}.'+ppid+'.txt';var sid='';try{{sid=fs.readFileSync(f,'utf8').trim();fs.unlinkSync(f)}}catch(e){{}}if(sid){{require('http').request({{hostname:'localhost',port:{},path:'/hook/cli',method:'POST',headers:{{'content-type':'application/json'}}}},r=>r.resume()).on('error',()=>{{}}).end(JSON.stringify({{title:'Approval timeout',body:'Approval took longer than expected.',cli_id:'{}',event_type:'posttooluse',session_id:sid,pid:ppid}}))}}}}catch(e){{}}""#,
        cli_id, port, cli_id,
    );

    GeneratedCommand { unix, windows: node }
}
