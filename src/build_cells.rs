use crate::log;
use crate::oops;
use crate::opt_str;
use crate::check_result;

use std::process::Command;

use toml::Value;

pub fn process(config: &Value) {
    let stage = "building cells";

    log!(stage, "reading configuration");

    let rust_features: [&str; 0] = [];

    let cargo = opt_str(config, &["cargo-path"]);
    let target = opt_str(config, &["build-cells", "cargo-target"]);
    let build_mode = opt_str(config, &["build-cells", "build-mode"]);
    let toolchain = opt_str(config, &["build-cells", "toolchain"]);

    let build_std_flags = [
        "-Z", "unstable-options",
        "-Z", "build-std=core,alloc",
        "-Z", "build-std-features=compiler-builtins-mem",
    ];

    let rust_flags = r#"
        --emit=obj
        -C debuginfo=2
        -C code-model=large
        -C relocation-model=static
        -D unused-must-use
        -Z merge-functions=disabled
        -Z share-generics=no
    "#;

    if !["debug", "release"].contains(&build_mode.as_str()) {
        oops!(stage, "build-mode must be \"debug\" or \"release\"");
    }

    log!(stage, "building all crates using cargo");

    let result = Command::new(cargo)
            .env("RUSTFLAGS", rust_flags)
            .arg(&format!("+{}", &toolchain))
            .arg("build")
            .arg("--manifest-path=kernel/Cargo.toml")
            .arg(&format!("--{}", &build_mode))
            .args(build_std_flags)
            .args(rust_features)
            .arg("--target")
            .arg(&format!("cfg/{}.json", &target))
            .status();

    check_result(stage, result, "cargo invocation failed");
}