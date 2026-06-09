use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub camera_available: bool,
    pub microphone_available: bool,
    pub screen_capture_available: bool,
    pub virtual_display_active: bool,
    pub virtual_display_resolution: String,
    pub audio_input_device: String,
    pub audio_output_device: String,
    pub video_input_device: String,
    pub ocr_available: bool,
    pub stt_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResult {
    pub success: bool,
    pub message: String,
    pub data_base64: Option<String>,
    pub mime_type: Option<String>,
    pub text: Option<String>,
}

pub struct DeviceManager;

impl DeviceManager {
    pub fn get_status(settings: &crate::settings::DeviceSettings) -> DeviceStatus {
        DeviceStatus {
            camera_available: settings.camera_enabled,
            microphone_available: settings.microphone_enabled,
            screen_capture_available: settings.screen_capture_enabled && screen_capture_supported(),
            virtual_display_active: settings.virtual_display_extend,
            virtual_display_resolution: settings.virtual_display_resolution.clone(),
            audio_input_device: settings.audio_input_device.clone(),
            audio_output_device: settings.audio_output_device.clone(),
            video_input_device: settings.video_input_device.clone(),
            ocr_available: tesseract_available(),
            stt_available: whisper_available(),
        }
    }

    pub fn capture_screen(settings: &crate::settings::DeviceSettings) -> CaptureResult {
        if !settings.screen_capture_enabled {
            return CaptureResult {
                success: false,
                message: "Захват экрана отключён в настройках".into(),
                data_base64: None,
                mime_type: None,
                text: None,
            };
        }

        match capture_primary_monitor_png() {
            Ok(png) => {
                let b64 = B64.encode(&png);
                let mut message = format!(
                    "Снимок экрана: {} байт",
                    png.len()
                );
                let mut text = None;
                if settings.ocr_on_images {
                    match ocr_png_bytes(&png) {
                        Ok(ocr) if !ocr.trim().is_empty() => {
                            text = Some(ocr.clone());
                            message.push_str(&format!(" | OCR: {} симв.", ocr.chars().count()));
                        }
                        Ok(_) => message.push_str(" | OCR: текст не найден"),
                        Err(e) => message.push_str(&format!(" | OCR: {e}")),
                    }
                }
                CaptureResult {
                    success: true,
                    message,
                    data_base64: Some(b64),
                    mime_type: Some("image/png".into()),
                    text,
                }
            }
            Err(e) => CaptureResult {
                success: false,
                message: format!("Ошибка захвата экрана: {e}"),
                data_base64: None,
                mime_type: None,
                text: None,
            },
        }
    }

    pub fn ocr_screen(settings: &crate::settings::DeviceSettings) -> CaptureResult {
        if !settings.screen_capture_enabled {
            return CaptureResult {
                success: false,
                message: "Захват экрана отключён".into(),
                data_base64: None,
                mime_type: None,
                text: None,
            };
        }
        let cap = Self::capture_screen(settings);
        if !cap.success {
            return cap;
        }
        if let Some(ref t) = cap.text {
            if !t.trim().is_empty() {
                return CaptureResult {
                    success: true,
                    message: format!("OCR: {} символов", t.chars().count()),
                    data_base64: cap.data_base64,
                    mime_type: cap.mime_type,
                    text: cap.text,
                };
            }
        }
        if let Some(ref b64) = cap.data_base64 {
            if let Ok(bytes) = B64.decode(b64) {
                match ocr_png_bytes(&bytes) {
                    Ok(ocr) => {
                        return CaptureResult {
                            success: true,
                            message: format!("OCR: {} символов", ocr.chars().count()),
                            data_base64: cap.data_base64,
                            mime_type: cap.mime_type,
                            text: Some(ocr),
                        };
                    }
                    Err(e) => {
                        return CaptureResult {
                            success: false,
                            message: e,
                            data_base64: cap.data_base64,
                            mime_type: cap.mime_type,
                            text: None,
                        };
                    }
                }
            }
        }
        CaptureResult {
            success: false,
            message: "OCR не удался".into(),
            data_base64: cap.data_base64,
            mime_type: cap.mime_type,
            text: None,
        }
    }

    pub fn capture_audio(settings: &crate::settings::DeviceSettings) -> CaptureResult {
        if !settings.microphone_enabled {
            return CaptureResult {
                success: false,
                message: "Микрофон отключён в настройках".into(),
                data_base64: None,
                mime_type: None,
                text: None,
            };
        }

        match record_audio_wav(3) {
            Ok(wav) => {
                let b64 = B64.encode(&wav);
                let mut message = format!(
                    "Аудио: {} байт WAV (~3 с), устройство '{}'",
                    wav.len(),
                    settings.audio_input_device
                );
                let mut text = None;
                if settings.transcribe_audio {
                    match transcribe_wav(&wav) {
                        Ok(t) if !t.trim().is_empty() => {
                            text = Some(t.clone());
                            message.push_str(&format!(" | STT: {} симв.", t.chars().count()));
                        }
                        Ok(_) => message.push_str(" | STT: пустой результат"),
                        Err(e) => message.push_str(&format!(" | STT: {e}")),
                    }
                }
                CaptureResult {
                    success: true,
                    message,
                    data_base64: Some(b64),
                    mime_type: Some("audio/wav".into()),
                    text,
                }
            }
            Err(e) => CaptureResult {
                success: false,
                message: format!(
                    "Запись аудио не удалась: {e}. Установите arecord (ALSA) или ffmpeg."
                ),
                data_base64: None,
                mime_type: None,
                text: None,
            },
        }
    }

    pub fn transcribe_audio(settings: &crate::settings::DeviceSettings) -> CaptureResult {
        let cap = Self::capture_audio(settings);
        if !cap.success {
            return cap;
        }
        if cap.text.as_ref().is_some_and(|t| !t.trim().is_empty()) {
            return cap;
        }
        if let Some(ref b64) = cap.data_base64 {
            if let Ok(bytes) = B64.decode(b64) {
                match transcribe_wav(&bytes) {
                    Ok(t) => {
                        return CaptureResult {
                            success: true,
                            message: format!("STT: {} символов", t.chars().count()),
                            data_base64: cap.data_base64,
                            mime_type: cap.mime_type,
                            text: Some(t),
                        };
                    }
                    Err(e) => {
                        return CaptureResult {
                            success: false,
                            message: e,
                            data_base64: cap.data_base64,
                            mime_type: cap.mime_type,
                            text: None,
                        };
                    }
                }
            }
        }
        CaptureResult {
            success: false,
            message: "STT не удался".into(),
            data_base64: cap.data_base64,
            mime_type: cap.mime_type,
            text: None,
        }
    }

    pub fn capture_camera(settings: &crate::settings::DeviceSettings) -> CaptureResult {
        if !settings.camera_enabled {
            return CaptureResult {
                success: false,
                message: "Камера отключена в настройках".into(),
                data_base64: None,
                mime_type: None,
                text: None,
            };
        }
        match capture_camera_frame() {
            Ok((bytes, mime)) => CaptureResult {
                success: true,
                message: format!(
                    "Кадр камеры '{}' @ {}fps — {} байт",
                    settings.video_input_device,
                    settings.frame_rate,
                    bytes.len()
                ),
                data_base64: Some(B64.encode(&bytes)),
                mime_type: Some(mime),
                text: None,
            },
            Err(e) => CaptureResult {
                success: false,
                message: format!(
                    "Камера: {e}. Попробуйте ffmpeg -f v4l2 (Linux) или включите устройство."
                ),
                data_base64: None,
                mime_type: None,
                text: None,
            },
        }
    }
}

fn screen_capture_supported() -> bool {
    command_available("grim")
        || command_available("scrot")
        || command_available("gnome-screenshot")
        || command_available("import")
        || command_available("ffmpeg")
}

fn command_available(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        || Command::new(cmd)
            .arg("-version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
}

fn capture_primary_monitor_png() -> Result<Vec<u8>, String> {
    let path = temp_path("screen.png");
    let path_s = path.to_str().unwrap_or("/tmp/silenium-screen.png");

    let attempts: &[(&str, Vec<&str>)] = &[
        ("grim", vec![path_s]),
        ("scrot", vec![path_s]),
        ("gnome-screenshot", vec!["-f", path_s]),
        ("import", vec!["-window", "root", path_s]),
        (
            "ffmpeg",
            vec![
                "-y",
                "-f",
                "x11grab",
                "-video_size",
                "1920x1080",
                "-i",
                ":0.0",
                "-frames:v",
                "1",
                path_s,
            ],
        ),
    ];

    for (cmd, args) in attempts {
        if !command_available(cmd) && *cmd != "ffmpeg" {
            continue;
        }
        let ok = Command::new(cmd)
            .args(args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            if let Ok(data) = std::fs::read(&path) {
                let _ = std::fs::remove_file(&path);
                if data.len() > 64 {
                    return Ok(data);
                }
            }
        }
    }

    Err(
        "Нет утилиты захвата (grim, scrot, gnome-screenshot, import или ffmpeg x11grab)".into(),
    )
}

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("silenium-{name}"))
}

fn tesseract_available() -> bool {
    Command::new("tesseract")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn whisper_available() -> bool {
    ["whisper", "whisper-cli", "main"]
        .iter()
        .any(|cmd| {
            Command::new(cmd)
                .arg("--help")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
}

fn ocr_png_bytes(png: &[u8]) -> Result<String, String> {
    if !tesseract_available() {
        return Err("tesseract не найден в PATH (sudo apt install tesseract-ocr tesseract-ocr-rus)".into());
    }
    let path = temp_path("ocr.png");
    std::fs::write(&path, png).map_err(|e| e.to_string())?;
    let out = Command::new("tesseract")
        .arg(&path)
        .arg("stdout")
        .arg("-l")
        .arg("eng+rus")
        .output()
        .map_err(|e| format!("tesseract: {e}"))?;
    let _ = std::fs::remove_file(&path);
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!("tesseract ошибка: {err}"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn record_audio_wav(seconds: u32) -> Result<Vec<u8>, String> {
    let path = temp_path("cap.wav");
    let _ = std::fs::remove_file(&path);

    if Command::new("arecord")
        .args([
            "-q",
            "-d",
            &seconds.to_string(),
            "-f",
            "cd",
            "-t",
            "wav",
            path.to_str().unwrap_or("/tmp/silenium-cap.wav"),
        ])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        let data = std::fs::read(&path).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&path);
        if data.len() > 44 {
            return Ok(data);
        }
    }

    if Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "alsa",
            "-i",
            "default",
            "-t",
            &seconds.to_string(),
            "-ac",
            "1",
            "-ar",
            "16000",
            path.to_str().unwrap_or("/tmp/silenium-cap.wav"),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        let data = std::fs::read(&path).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&path);
        if data.len() > 44 {
            return Ok(data);
        }
    }

    Err("arecord/ffmpeg недоступны или нет входного устройства".into())
}

fn transcribe_wav(wav: &[u8]) -> Result<String, String> {
    let path = temp_path("stt.wav");
    std::fs::write(&path, wav).map_err(|e| e.to_string())?;
    let out_txt = temp_path("stt.txt");

    for (cmd, args) in [
        (
            "whisper",
            vec![
                path.to_str().unwrap_or(""),
                "-m",
                "tiny",
                "-l",
                "ru",
                "-otxt",
                "-of",
                out_txt.with_extension("").to_str().unwrap_or("/tmp/silenium-stt"),
            ],
        ),
        (
            "whisper-cli",
            vec![
                "-m",
                "models/ggml-tiny.bin",
                "-f",
                path.to_str().unwrap_or(""),
                "-l",
                "ru",
                "-otxt",
            ],
        ),
    ] {
        if Command::new(cmd)
            .args(&args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            let txt_path = if out_txt.exists() {
                out_txt.clone()
            } else {
                path.with_extension("txt")
            };
            if txt_path.exists() {
                let text = std::fs::read_to_string(&txt_path).unwrap_or_default();
                let _ = std::fs::remove_file(&txt_path);
                let _ = std::fs::remove_file(&path);
                if !text.trim().is_empty() {
                    return Ok(text.trim().to_string());
                }
            }
        }
    }

    let _ = std::fs::remove_file(&path);
    Err("whisper/whisper-cli не найден или модель недоступна".into())
}

fn capture_camera_frame() -> Result<(Vec<u8>, String), String> {
    let path = temp_path("cam.jpg");
    if Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "v4l2",
            "-i",
            "/dev/video0",
            "-frames:v",
            "1",
            "-q:v",
            "2",
            path.to_str().unwrap_or("/tmp/silenium-cam.jpg"),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        let data = std::fs::read(&path).map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&path);
        if !data.is_empty() {
            return Ok((data, "image/jpeg".into()));
        }
    }
    Err("ffmpeg v4l2 /dev/video0 недоступен".into())
}
