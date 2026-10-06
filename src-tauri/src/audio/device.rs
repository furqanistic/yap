//! Listing input devices and resolving the one chosen in settings.

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

use super::AudioError;

/// The settings value meaning "follow the system default microphone".
pub const DEFAULT_DEVICE: &str = "default";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputDevice {
    /// Stable across restarts and reconnects (cpal's device id).
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

fn device_id(device: &cpal::Device) -> Option<String> {
    device.id().ok().map(|id| id.to_string())
}

/// Every input device the OS reports. Works without any permission prompt.
pub fn list() -> Result<Vec<InputDevice>, AudioError> {
    let host = cpal::default_host();
    let default_id = host.default_input_device().as_ref().and_then(device_id);
    let mut devices: Vec<InputDevice> = host
        .input_devices()?
        .filter_map(|device| {
            let id = device_id(&device)?;
            Some(InputDevice {
                is_default: Some(&id) == default_id.as_ref(),
                name: device.to_string(),
                id,
            })
        })
        .collect();
    devices.sort_by(|a, b| b.is_default.cmp(&a.is_default).then(a.name.cmp(&b.name)));
    Ok(devices)
}

/// The device for a settings value. Falls back to the system default when
/// the value is `"default"` or the device is no longer connected.
pub fn resolve(id: &str) -> Result<cpal::Device, AudioError> {
    let host = cpal::default_host();
    if id != DEFAULT_DEVICE {
        let found = id
            .parse::<cpal::DeviceId>()
            .ok()
            .and_then(|parsed| host.device_by_id(&parsed))
            .filter(|device| device.supports_input());
        match found {
            Some(device) => return Ok(device),
            None => log::warn!("microphone {id} not found; using the default"),
        }
    }
    host.default_input_device().ok_or(AudioError::NoDevice)
}
