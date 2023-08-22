mod nix;

fn main() {
    let url = "github:NixOS/nixpkgs";
    let flake_ref = nix::ffi::parse_flakeref(url);
    let flake_ref_str = nix::ffi::flakeref_to_string(flake_ref);
    println!("{}", flake_ref_str);
}
