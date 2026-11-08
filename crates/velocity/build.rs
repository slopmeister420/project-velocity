// hybrid CRT on msvc: vcruntime linked statically, the UCRT stays the Windows system one (api-ms-win-crt-*),
// so the exe needs no Visual C++ redistributable next to it.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-bins=/NODEFAULTLIB:vcruntime.lib");
        println!("cargo:rustc-link-arg-bins=libvcruntime.lib");
    }
}
