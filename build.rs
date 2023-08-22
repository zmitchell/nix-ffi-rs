fn main() {
    cxx_build::bridge("src/nix.rs")
        .file("src/nix.cpp")
        .flag_if_supported("-std=c++2a")
        .includes(pkg_config::probe_library("nix-main").unwrap().include_paths)
        .compile("nix_ffi_rs");
    println!("cargo:rustc-link-lib=nixfetchers");

    println!("cargo:rerun-if-changed=include/nix.h");
    println!("cargo:rerun-if-changed=src/nix.cpp");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/nix.rs");

    pkg_config::probe_library("nix-main").unwrap();
    pkg_config::probe_library("nix-cmd").unwrap();
    pkg_config::probe_library("nix-expr").unwrap();
    pkg_config::probe_library("nix-store").unwrap();
}
