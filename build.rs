fn main() {
    // 在 Windows 上嵌入 manifest 文件
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");

        // 如果图标文件存在才设置
        if std::path::Path::new("icon.ico").exists() {
            res.set_icon("icon.ico");
            println!("cargo:rerun-if-changed=icon.ico");
        }

        println!("cargo:rerun-if-changed=app.manifest");
        res.compile().expect("Failed to embed administrator manifest");
    }
}
