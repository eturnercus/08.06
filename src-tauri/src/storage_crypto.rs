pub fn encrypt_at_rest(data: &str, enabled: bool) -> String {
    if !enabled {
        return data.to_string();
    }
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data.as_bytes())
}

pub fn decrypt_at_rest(data: &str, enabled: bool) -> String {
    if !enabled {
        return data.to_string();
    }
    use base64::Engine;
    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data.trim()) {
        if let Ok(s) = String::from_utf8(bytes) {
            return s;
        }
    }
    data.to_string()
}
