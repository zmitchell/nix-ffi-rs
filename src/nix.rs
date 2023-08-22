#[cxx::bridge(namespace = "nix_ffi_rs")]
pub(crate) mod ffi {
    unsafe extern "C++" {
        include!("nix-ffi-rs/include/nix.h");

        type FlakeRef;

        fn parse_flakeref(url: &str) -> UniquePtr<FlakeRef>;
        fn flakeref_to_string(flakeref: UniquePtr<FlakeRef>) -> String;
    }
}
