//! Compatibilidad entre Noctalia y Umbriel: includes y claves renombradas.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use toml_edit::DocumentMut;

use crate::env;

/// Claves que Noctalia escribe y Umbriel pudo renombrar.
const ALIASES: &[(&str, &str)] = &[("title_bar", "tab_bar")];

fn umbriel_config_path() -> PathBuf {
    env::app_config("umbriel").join("config.toml")
}

fn umbriel_theme_path() -> PathBuf {
    env::app_config("umbriel").join("noctalia.toml")
}

/// Rutas de configuración que acepta el Umbriel instalado.
pub fn umbriel_schema_paths() -> Vec<String> {
    let Some(raw) = env::capture("umbriel", &["config", "schema", "--json"]) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Vec::new();
    };
    value
        .get("options")
        .and_then(|options| options.as_array())
        .map(|options| {
            options
                .iter()
                .filter_map(|option| option.get("path").and_then(|p| p.as_str()))
                .map(|path| path.to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn canonical_key(path: &Path, base: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    absolute
        .canonicalize()
        .unwrap_or(absolute)
        .display()
        .to_string()
}

/// Detecta entradas repetidas en `[include.optional].files`.
pub fn duplicate_includes() -> Option<String> {
    let path = umbriel_config_path();
    let text = std::fs::read_to_string(&path).ok()?;
    let doc: DocumentMut = text.parse().ok()?;
    let base = path.parent().unwrap_or(Path::new("."));
    let files = doc
        .get("include")
        .and_then(|i| i.as_table_like())
        .and_then(|t| t.get("optional"))
        .and_then(|i| i.as_table_like())
        .and_then(|t| t.get("files"))
        .and_then(|i| i.as_array())?;

    let mut seen: Vec<String> = Vec::new();
    let mut duplicates: Vec<String> = Vec::new();
    for value in files.iter() {
        let Some(item) = value.as_str() else {
            continue;
        };
        let key = canonical_key(Path::new(item), base);
        if seen.contains(&key) {
            duplicates.push(item.to_string());
        } else {
            seen.push(key);
        }
    }

    if duplicates.is_empty() {
        None
    } else {
        Some(format!("includes repetidos: {}", duplicates.join(", ")))
    }
}

/// Elimina includes repetidos conservando la forma más corta (normalmente relativa).
pub fn normalize_umbriel_includes() -> Result<Option<String>> {
    let path = umbriel_config_path();
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("no se pudo leer {}", path.display()))?;
    let mut doc: DocumentMut = text
        .parse()
        .with_context(|| format!("{} no es TOML válido", path.display()))?;
    let base = path.parent().unwrap_or(Path::new(".")).to_path_buf();

    let Some(files) = doc
        .get_mut("include")
        .and_then(|i| i.as_table_like_mut())
        .and_then(|t| t.get_mut("optional"))
        .and_then(|i| i.as_table_like_mut())
        .and_then(|t| t.get_mut("files"))
        .and_then(|i| i.as_array_mut())
    else {
        return Ok(None);
    };

    let originals: Vec<String> = files
        .iter()
        .filter_map(|value| value.as_str().map(|s| s.to_string()))
        .collect();

    let mut order: Vec<String> = Vec::new();
    let mut best: Vec<(String, String)> = Vec::new();
    for item in &originals {
        let key = canonical_key(Path::new(item), &base);
        if let Some(entry) = best.iter_mut().find(|(k, _)| *k == key) {
            if item.len() < entry.1.len() {
                entry.1 = item.clone();
            }
        } else {
            order.push(key.clone());
            best.push((key, item.clone()));
        }
    }

    let normalized: Vec<String> = order
        .iter()
        .map(|key| {
            best.iter()
                .find(|(k, _)| k == key)
                .map(|(_, value)| value.clone())
                .unwrap_or_default()
        })
        .collect();

    if normalized == originals {
        return Ok(None);
    }

    files.clear();
    for value in &normalized {
        files.push(value.as_str());
    }

    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("no se pudo escribir {}", path.display()))?;
    Ok(Some(format!(
        "includes normalizados: {}",
        normalized.join(", ")
    )))
}

fn theme_alias_issues(paths: &[String]) -> Vec<(String, String)> {
    let path = umbriel_theme_path();
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(doc) = text.parse::<DocumentMut>() else {
        return Vec::new();
    };
    let Some(colors) = doc.get("colors").and_then(|i| i.as_table_like()) else {
        return Vec::new();
    };

    let mut issues = Vec::new();
    for (from, to) in ALIASES {
        let from_path = format!("colors.{from}");
        let to_path = format!("colors.{to}");
        let has_from = paths.iter().any(|p| p == &from_path);
        let has_to = paths.iter().any(|p| p == &to_path);
        if !has_from && has_to && colors.contains_key(from) {
            issues.push((from_path, to_path));
        }
    }
    issues
}

/// Detecta claves del tema generado que el Umbriel instalado no acepta.
pub fn umbriel_theme_issue() -> Option<String> {
    let paths = umbriel_schema_paths();
    if paths.is_empty() {
        return None;
    }
    let issues = theme_alias_issues(&paths);
    if issues.is_empty() {
        return None;
    }
    Some(
        issues
            .into_iter()
            .map(|(from, to)| format!("{from} debería ser {to}"))
            .collect::<Vec<_>>()
            .join("; "),
    )
}

/// Renombra en el tema generado las claves que Umbriel ya no acepta.
pub fn normalize_umbriel_theme() -> Result<Option<String>> {
    let paths = umbriel_schema_paths();
    if paths.is_empty() {
        return Ok(None);
    }
    let issues = theme_alias_issues(&paths);
    if issues.is_empty() {
        return Ok(None);
    }

    let path = umbriel_theme_path();
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("no se pudo leer {}", path.display()))?;
    let mut doc: DocumentMut = text
        .parse()
        .with_context(|| format!("{} no es TOML válido", path.display()))?;

    let Some(colors) = doc.get_mut("colors").and_then(|i| i.as_table_like_mut()) else {
        return Ok(None);
    };

    let mut renamed = Vec::new();
    for (from, to) in ALIASES {
        if !colors.contains_key(from) {
            continue;
        }
        if colors.contains_key(to) {
            // La clave válida ya existe: solo hay que retirar la obsoleta.
            colors.remove(from);
            renamed.push(format!(
                "colors.{from} eliminada (duplicada de colors.{to})"
            ));
        } else if let Some(item) = colors.remove(from) {
            colors.insert(to, item);
            renamed.push(format!("colors.{from} → colors.{to}"));
        }
    }

    if renamed.is_empty() {
        return Ok(None);
    }

    std::fs::write(&path, doc.to_string())
        .with_context(|| format!("no se pudo escribir {}", path.display()))?;
    Ok(Some(renamed.join(", ")))
}

/// Aplica todas las correcciones de compatibilidad y devuelve las aplicadas.
pub fn fix_all() -> Vec<String> {
    let mut fixed = Vec::new();
    if let Ok(Some(message)) = normalize_umbriel_includes() {
        fixed.push(message);
    }
    if let Ok(Some(message)) = normalize_umbriel_theme() {
        fixed.push(message);
    }
    fixed
}
