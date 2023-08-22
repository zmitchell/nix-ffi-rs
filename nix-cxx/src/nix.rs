#[cxx::bridge(namespace = "nix_cxx")]
pub(crate) mod ffi {
    unsafe extern "C++" {
        include!("nix-cxx/include/nix.h");

        type FlakeRef;

        fn parse_flakeref(url: &str) -> UniquePtr<FlakeRef>;
        fn flakeref_to_string(flakeref: UniquePtr<FlakeRef>) -> String;
    }
}
