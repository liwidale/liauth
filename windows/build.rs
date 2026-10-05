fn main() {
    #[cfg(windows)]
    {
        let icon = std::path::Path::new("../branding/icon.ico");
        let mut resource = winresource::WindowsResource::new();
        resource.set("ProductName", "LiAuth");
        resource.set("FileDescription", "LiAuth Authenticator");
        resource.set("LegalCopyright", "MIT License, liwidale");
        if icon.exists() {
            resource.set_icon(icon.to_str().unwrap());
        }
        let _ = resource.compile();
    }
}
