#[path = "build_support/localization.rs"]
mod localization;

fn main() {
    localization::generate();
    // 在 Windows 上嵌入 manifest 文件
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");

        // The lowest resource ID is the static green application/shortcut icon.
        // Runtime windows and the tray select another embedded color after readback.
        for (id, name) in [
            ("1", "app"),
            ("2", "balanced"),
            ("3", "performance"),
            ("4", "unknown"),
        ] {
            let path = format!("assets/icons/{name}.ico");
            res.set_icon_with_id(&path, id);
            println!("cargo:rerun-if-changed={path}");
        }

        println!("cargo:rerun-if-changed=app.manifest");
        res.compile()
            .expect("Failed to embed administrator manifest");
    }
}
