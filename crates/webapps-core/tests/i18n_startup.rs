use gettextrs::gettext;
use std::{fs, path::Path, process::Command};

fn main() {
    if std::env::args().any(|arg| arg == "--ignored") {
        return;
    }
    if std::env::args().any(|arg| arg == "--list") {
        println!("i18n_startup: test");
        return;
    }
    let prefix = tempfile::tempdir().expect("temporary locale prefix");
    compile_catalog(prefix.path());

    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("LANGUAGE", "pt_BR");
    std::env::set_var("BIGLINUX_WEBAPPS_PREFIX", prefix.path());
    // SAFETY: This standalone test has no test-harness threads or signal handlers.
    unsafe { webapps_core::i18n::init() };

    assert_eq!(["Name", "Save"].map(gettext), ["Nome", "Salvar"]);
    let translated = std::thread::spawn(|| ["Name", "Save"].map(gettext))
        .join()
        .expect("translation thread");
    assert_eq!(translated, ["Nome", "Salvar"]);
    println!("i18n startup and worker translations passed");
}

fn compile_catalog(prefix: &Path) {
    let catalog_dir = prefix.join("share/locale/pt_BR/LC_MESSAGES");
    fs::create_dir_all(&catalog_dir).expect("locale directory");
    let catalog = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../po/pt-BR.po");
    let status = Command::new("msgfmt")
        .arg("--check")
        .arg(catalog)
        .arg("-o")
        .arg(catalog_dir.join("biglinux-webapps.mo"))
        .status()
        .expect("msgfmt must be installed");
    assert!(status.success(), "compile the Portuguese catalog");
}
