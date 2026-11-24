//! Servidor LAN para sincronizar tema y wallpaper con el launcher Android.
//!
//! Expone `/info`, `/theme` y `/wallpaper` protegidos por un token. Pensado para
//! la red local: no uses este servidor en una red pública.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::env;
use crate::theme::Theme;

pub struct ServeOptions {
    pub port: u16,
    pub bind: String,
    pub token: Option<String>,
    pub theme: Option<PathBuf>,
    pub wallpaper: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
struct Info {
    name: String,
    scheme: String,
    mode: String,
    wallpaper: bool,
    theme_bytes: usize,
}

fn random_token() -> String {
    let mut bytes = [0u8; 4];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    } else {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        bytes = (nanos as u32).to_le_bytes();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip().to_string())
}

fn theme_json(path: Option<&Path>) -> Result<String> {
    if let Some(path) = path {
        if !path.exists() {
            bail!("no existe el tema {}", path.display());
        }
        return std::fs::read_to_string(path)
            .with_context(|| format!("no se pudo leer {}", path.display()));
    }

    // Intenta leer el tema activo de Noctalia; si no está disponible, reutiliza
    // la última exportación del servidor.
    let cache = env::app_config("noct").join("sync-theme.json");
    match Theme::from_noctalia(None, None) {
        Ok(theme) => {
            let json = theme.to_json()?;
            if let Some(parent) = cache.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&cache, &json);
            Ok(json)
        }
        Err(err) => {
            if cache.exists() {
                std::fs::read_to_string(&cache)
                    .with_context(|| format!("no se pudo leer {}", cache.display()))
            } else {
                Err(err).context(
                    "no se pudo leer el tema de Noctalia; exporta uno con \
                     `noct theme export --out ~/.config/noct/sync-theme.json` \
                     o pasa --theme <archivo>",
                )
            }
        }
    }
}

fn wallpaper_path(theme: &str, explicit: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = explicit {
        return path.is_file().then(|| path.to_path_buf());
    }
    let value: serde_json::Value = serde_json::from_str(theme).ok()?;
    let desktop = value
        .get("wallpapers")
        .and_then(|w| w.get("desktop"))
        .and_then(|d| d.as_str())?;
    let path = PathBuf::from(desktop);
    path.is_file().then_some(path)
}

fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    }
}

fn query_token(path: &str) -> Option<String> {
    let (_, query) = path.split_once('?')?;
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == "token").then(|| value.to_string())
    })
}

fn url_decode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = &value[index + 1..index + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte as char);
                    index += 3;
                    continue;
                }
                out.push('%');
            }
            b'+' => out.push(' '),
            byte => out.push(byte as char),
        }
        index += 1;
    }
    out
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) -> Result<()> {
    let header = format!(
        "HTTP/1.1 {status}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Cache-Control: no-store\r\n\
         Connection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()?;
    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    token: &str,
    theme: &str,
    wallpaper: Option<&Path>,
    info: &Info,
) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts.next().unwrap_or_default();

    // Consume headers.
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line == "\r\n" || line == "\n" {
            break;
        }
    }

    if method != "GET" {
        return respond(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain; charset=utf-8",
            b"method not allowed",
        );
    }

    let given = query_token(path)
        .map(|value| url_decode(&value))
        .unwrap_or_default();
    if given != token {
        return respond(
            &mut stream,
            "401 Unauthorized",
            "application/json",
            r#"{"error":"token inválido"}"#.as_bytes(),
        );
    }

    let route = path.split('?').next().unwrap_or(path);
    match route {
        "/info" => {
            let body = serde_json::to_vec(info)?;
            respond(&mut stream, "200 OK", "application/json", &body)
        }
        "/theme" => respond(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            theme.as_bytes(),
        ),
        "/wallpaper" => match wallpaper {
            Some(path) => {
                let bytes = std::fs::read(path)
                    .with_context(|| format!("no se pudo leer {}", path.display()))?;
                respond(&mut stream, "200 OK", content_type(path), &bytes)
            }
            None => respond(
                &mut stream,
                "404 Not Found",
                "application/json",
                br#"{"error":"sin wallpaper"}"#,
            ),
        },
        _ => respond(
            &mut stream,
            "404 Not Found",
            "application/json",
            br#"{"error":"ruta desconocida"}"#,
        ),
    }
}

pub fn serve(options: ServeOptions) -> Result<()> {
    let token = options.token.unwrap_or_else(random_token);
    let theme = theme_json(options.theme.as_deref())?;
    let wallpaper = wallpaper_path(&theme, options.wallpaper.as_deref());

    let value: serde_json::Value = serde_json::from_str(&theme).unwrap_or_default();
    let info = Info {
        name: value
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Noct")
            .to_string(),
        scheme: value
            .get("scheme")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        mode: value
            .get("resolved_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("dark")
            .to_string(),
        wallpaper: wallpaper.is_some(),
        theme_bytes: theme.len(),
    };

    let address = format!("{}:{}", options.bind, options.port);
    let listener = TcpListener::bind(&address)
        .with_context(|| format!("no se pudo escuchar en {address}; ¿el puerto está ocupado?"))?;

    println!("Noct sync escuchando en {address}");
    if let Some(ip) = local_ip() {
        println!("  URL:   http://{ip}:{}", options.port);
    } else {
        println!("  URL:   http://<ip-del-escritorio>:{}", options.port);
    }
    println!("  token: {token}");
    println!("  tema:  {} · {}", info.name, info.mode);
    println!(
        "  wallpaper: {}",
        wallpaper
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "no incluido".to_string())
    );
    println!("\nEn el launcher Android, abre Sincronizar y pega URL + token.");

    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue;
        };
        let token = token.clone();
        let theme = theme.clone();
        let wallpaper = wallpaper.clone();
        let info = info.clone();
        thread::spawn(move || {
            let _ = handle_client(stream, &token, &theme, wallpaper.as_deref(), &info);
        });
    }
    Ok(())
}
