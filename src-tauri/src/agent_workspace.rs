use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn resolve_in_workspace(workspace: &str, rel: &str) -> Result<PathBuf, String> {
    let root = PathBuf::from(workspace.trim());
    if root.as_os_str().is_empty() {
        return Err("Рабочая папка чата не задана".into());
    }
    if !root.is_absolute() {
        return Err("Рабочая папка должна быть абсолютным путём".into());
    }
    let root = fs::canonicalize(&root).map_err(|e| format!("Папка недоступна: {e}"))?;
    if !root.is_dir() {
        return Err("Рабочая папка не существует".into());
    }

    let rel = rel.trim().trim_start_matches(['/', '\\']);
    let mut target = root.clone();
    for part in Path::new(rel).components() {
        match part {
            Component::Normal(p) => target.push(p),
            Component::ParentDir => {
                return Err("Выход за пределы рабочей папки запрещён".into());
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("Абсолютные пути внутри workspace запрещены".into());
            }
            Component::CurDir => {}
        }
    }

    let canonical = if target.exists() {
        fs::canonicalize(&target).map_err(|e| e.to_string())?
    } else {
        let mut check = root.clone();
        for part in Path::new(rel).components() {
            if let Component::Normal(p) = part {
                check.push(p);
            }
        }
        if !check.starts_with(&root) {
            return Err("Путь вне рабочей папки".into());
        }
        check
    };

    if !canonical.starts_with(&root) {
        return Err("Доступ только внутри рабочей папки чата".into());
    }
    Ok(canonical)
}

pub fn read_file(workspace: &str, rel: &str, max_bytes: usize) -> Result<String, String> {
    let path = resolve_in_workspace(workspace, rel)?;
    if !path.is_file() {
        return Err(format!("Не файл: {}", path.display()));
    }
    let data = fs::read(&path).map_err(|e| e.to_string())?;
    if data.len() > max_bytes {
        return Err(format!("Файл больше {max_bytes} байт"));
    }
    String::from_utf8(data).map_err(|_| "Файл не UTF-8".into())
}

pub fn write_file(workspace: &str, rel: &str, content: &str) -> Result<String, String> {
    let path = resolve_in_workspace(workspace, rel)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, content.as_bytes()).map_err(|e| e.to_string())?;
    Ok(format!("Записано {} байт → {}", content.len(), path.display()))
}

pub fn list_dir(workspace: &str, rel: &str) -> Result<Vec<String>, String> {
    let path = if rel.trim().is_empty() {
        resolve_in_workspace(workspace, ".")?
    } else {
        resolve_in_workspace(workspace, rel)?
    };
    if !path.is_dir() {
        return Err("Не папка".into());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let suffix = if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            "/"
        } else {
            ""
        };
        names.push(format!("{name}{suffix}"));
    }
    names.sort();
    Ok(names)
}

pub fn extract_file_path_from_prompt(prompt: &str) -> Option<String> {
    for token in prompt.split_whitespace() {
        let t = token.trim_matches(|c: char| "\"'()[]{}<>,".contains(c));
        if t.contains('.') && !t.starts_with("http") && t.len() < 260 {
            return Some(t.to_string());
        }
    }
    None
}
