nix-shell -E '
let
  unstable = import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/nixos-unstable.tar.gz";
  }) {};
in
  (import <nixpkgs> {}).mkShell {
    buildInputs = [ unstable.code-cursor ];
  }
'
