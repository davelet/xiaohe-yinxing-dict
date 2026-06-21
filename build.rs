fn main() {
    // Only embed icon on Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("AppIcon.ico");
        res.set_language(0x0804); // Chinese Simplified
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Failed to compile Windows resource: {e}");
        }
    }
}
