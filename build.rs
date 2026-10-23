fn main() {
    slint_build::compile("ui/app.slint").expect("GUI Slint non valida");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let lib =
            std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("native");
        println!("cargo:rustc-link-search=native={}", lib.display());
        winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .set("ProductName", "AmbraMusic")
            .set("FileDescription", "AmbraMusic")
            .set("LegalCopyright", "GPL-3.0-or-later")
            .compile()
            .expect("icona Windows");
    }
}
