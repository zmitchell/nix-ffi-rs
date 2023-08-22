#include "nix-ffi-rs/include/nix.h"
#include <memory>

using namespace std;

namespace nix_ffi_rs
{
  std::unique_ptr<nix::FlakeRef> parse_flakeref(rust::Str url)
  {
    // std::unique_ptr<nix::FlakeRef> originalRef = std::make_unique<nix::FlakeRef>(nix::parseFlakeRef(string(url), nix::absPath(".")));
    // std::unique_ptr<nix::FlakeRef> parsed = std::unique_ptr(new nix::FlakeRef( std::move(originalFlake)));
    nix::FlakeRef originalRef = nix::parseFlakeRef(string(url), nix::absPath("."));
    return std::make_unique<nix::FlakeRef>(originalRef);
  }

  rust::String flakeref_to_string(std::unique_ptr<nix::FlakeRef> flakeref)
  {
    auto s = flakeref->to_string();
    return rust::String(s);
  }
}
