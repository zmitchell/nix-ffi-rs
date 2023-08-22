#include "nix-cxx/include/nix.h"
#include <memory>

using namespace std;

namespace nix_cxx
{
  std::unique_ptr<nix::FlakeRef> parse_flakeref(rust::Str url)
  {
    nix::FlakeRef originalRef = nix::parseFlakeRef(string(url), nix::absPath("."));
    return std::make_unique<nix::FlakeRef>(originalRef);
  }

  rust::String flakeref_to_string(std::unique_ptr<nix::FlakeRef> flakeref)
  {
    auto s = flakeref->to_string();
    return rust::String(s);
  }
}
