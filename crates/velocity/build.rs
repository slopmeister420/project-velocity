// hybrid CRT on msvc: vcruntime linked statically, the UCRT stays the Windows system one (api-ms-win-crt-*),
// so the exe needs no Visual C++ redistributable next to it.
fn main() {
    // short git hash for crash.log (unknown outside a checkout)
    if let Ok(o) = std::process::Command::new("git").args(["rev-parse", "--short", "HEAD"]).output() {
        let h = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if o.status.success() && !h.is_empty() {
            println!("cargo:rustc-env=VELOCITY_GIT_HASH={h}");
        }
    }
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-bins=/NODEFAULTLIB:vcruntime.lib");
        println!("cargo:rustc-link-arg-bins=libvcruntime.lib");
    }
}
