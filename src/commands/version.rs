pub fn run_version() {
    let version = env!("CARGO_PKG_VERSION");
    let target = if cfg!(target_arch = "x86_64") {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-gnu"
    } else {
        "unknown-architecture"
    };

    println!("dockture v{}", version);
    println!("Target Architecture: {}", target);
}
