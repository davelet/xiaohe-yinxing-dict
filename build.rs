fn main() {
    // Only embed icon on Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("AppIcon.ico");
        res.set_language(0x0804); // Chinese Simplified
        // 显式声明 asInvoker：否则无 manifest 的 updater.exe 会被 Windows UAC
        // 的"安装程序检测"启发式规则（文件名含 update/install/setup）误判为安装程序，
        // 导致 CreateProcess 报 ERROR_ELEVATION_REQUIRED (os error 740)，自动更新失败。
        res.set_manifest(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges xmlns="urn:schemas-microsoft-com:asm.v3">
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#,
        );
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Failed to compile Windows resource: {e}");
        }
    }
}
