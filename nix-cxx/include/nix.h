#pragma once

#include "rust/cxx.h"
#include <nix/flake/flake.hh>
#include <nix/fetchers.hh>
#include <nix/error.hh>

namespace nix_cxx
{
  using nix::FlakeRef;

  std::unique_ptr<nix::FlakeRef> parse_flakeref(rust::Str url);
  rust::String flakeref_to_string(std::unique_ptr<nix::FlakeRef> flakeref);
}


