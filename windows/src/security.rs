use liauth_crypto::{random_key, SymmetricKey};

const SERVICE: &str = "LiAuth";
const QUICK_UNLOCK_ENTRY: &str = "quick-unlock-key";
pub const QUICK_UNLOCK_SLOT: &str = "device";

pub fn apply_capture_protection(enabled: bool) {
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;
        use windows::Win32::UI::WindowsAndMessaging::{
            SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
        };
        let hwnd: HWND = GetActiveWindow();
        if !hwnd.is_invalid() {
            let affinity = if enabled { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
            let _ = SetWindowDisplayAffinity(hwnd, affinity);
        }
    }
    #[cfg(not(windows))]
    let _ = enabled;
}

pub fn store_quick_unlock_key() -> Option<SymmetricKey> {
    let key = random_key();
    let encoded = liauth_crypto::encode_base64(key.bytes());
    let entry = keyring::Entry::new(SERVICE, QUICK_UNLOCK_ENTRY).ok()?;
    entry.set_password(&encoded).ok()?;
    Some(key)
}

pub fn load_quick_unlock_key() -> Option<SymmetricKey> {
    let entry = keyring::Entry::new(SERVICE, QUICK_UNLOCK_ENTRY).ok()?;
    let encoded = entry.get_password().ok()?;
    let bytes = liauth_crypto::decode_base64(&encoded).ok()?;
    SymmetricKey::from_slice(&bytes).ok()
}

pub fn clear_quick_unlock_key() {
    if let Ok(entry) = keyring::Entry::new(SERVICE, QUICK_UNLOCK_ENTRY) {
        let _ = entry.delete_credential();
    }
}
