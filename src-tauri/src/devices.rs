use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub camera_available: bool,
    pub microphone_available: bool,
    pub screen_capture_available: bool,
    pub virtual_display_active: bool,
    pub virtual_display_resolution: String,
    pub audio_input: String,
    pub audio_output: String,
    pub video_input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureResult {
    pub success: bool,
    pub message: String,
    pub data_base64: Option<String>,
    pub mime_type: Option<String>,
}

pub struct DeviceManager;

impl DeviceManager {
    pub fn get_status(settings: &crate::settings::DeviceSettings) -> DeviceStatus {
        DeviceStatus {
            camera_available: settings.camera_enabled,
            microphone_available: settings.microphone_enabled,
            screen_capture_available: settings.screen_capture_enabled,
            virtual_display_active: settings.virtual_display_extend,
            virtual_display_resolution: settings.virtual_display_resolution.clone(),
            audio_input: settings.audio_input_device.clone(),
            audio_output: settings.audio_output_device.clone(),
            video_input: settings.video_input_device.clone(),
        }
    }

    pub fn capture_screen(settings: &crate::settings::DeviceSettings) -> CaptureResult {
        if !settings.screen_capture_enabled {
            return CaptureResult {
                success: false,
                message: "Захват экрана отключён в настройках".into(),
                data_base64: None,
                mime_type: None,
            };
        }
        CaptureResult {
            success: true,
            message: format!(
                "Виртуальный дисплей: {} @ {}",
                if settings.virtual_display_extend {
                    "расширение"
                } else {
                    "зеркало"
                },
                settings.virtual_display_resolution
            ),
            data_base64: None,
            mime_type: Some("image/png".into()),
        }
    }

    pub fn capture_audio(settings: &crate::settings::DeviceSettings) -> CaptureResult {
        if !settings.microphone_enabled {
            return CaptureResult {
                success: false,
                message: "Микрофон отключён в настройках".into(),
                data_base64: None,
                mime_type: None,
            };
        }
        CaptureResult {
            success: true,
            message: format!(
                "Аудио с устройства '{}' @ {}Hz",
                settings.audio_input_device, settings.frame_rate
            ),
            data_base64: None,
            mime_type: Some("audio/wav".into()),
        }
    }

    pub fn capture_camera(settings: &crate::settings::DeviceSettings) -> CaptureResult {
        if !settings.camera_enabled {
            return CaptureResult {
                success: false,
                message: "Камера отключена в настройках".into(),
                data_base64: None,
                mime_type: None,
            };
        }
        CaptureResult {
            success: true,
            message: format!(
                "Камера '{}' @ {}fps",
                settings.video_input_device, settings.frame_rate
            ),
            data_base64: None,
            mime_type: Some("image/jpeg".into()),
        }
    }
}
