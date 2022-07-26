use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/helpers.c");
    let mut build = cc::Build::new();
    build.warnings(true)
        .define(
            &format!("CFG_TARGET_OS_{}", env::var("CARGO_CFG_TARGET_OS").unwrap()),
            None,
        )
        .file("src/helpers.c");
    
        if env::var("CARGO_CFG_TARGET_OS").unwrap().contains("theseus") {
            build
                .use_plt(false)
                .pic(false)
                // Theseus doesn't yet support the GOT (Global Offset Table),
                // so we disable usage of the GOT by forcing no PLT and PIC.
                // In Rust, this is accomplished using `-C relocation-model=static`.
                .flag("-fno-plt")
                .flag("-fno-pic")
                // Theseus currently uses the large code model.
                .flag("-mcmodel=large") 
                // Disable stack smashing protection
                .flag("-fno-stack-protector");

                // eprintln!("Theseus build: {:?}\n{:?}", build, build.get_compiler());
        }

    build.compile("wasmtime-helpers");
}
