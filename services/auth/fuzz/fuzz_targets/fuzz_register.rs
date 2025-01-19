#![no_main]
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    // Limit input size
    if data.len() < 10 || data.len() > 1024 {
        return;
    }

    // Try to parse as UTF-8
    if let Ok(input_str) = std::str::from_utf8(data) {
        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<Value>(input_str) {
            // Check if it's an object
            if let Some(obj) = json.as_object() {
                // Check required fields
                let _ = obj.get("username");
                let _ = obj.get("email");
                let _ = obj.get("password");
            }
        }
    }
});
