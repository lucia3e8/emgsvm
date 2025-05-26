let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };
in
with pkgs;
mkShell {
  buildInputs = with pkgs; [
    cargo
    pkg-config
    udev
    xorg.libxcb
    xorg.libX11
    xorg.libXrandr
    xorg.libXinerama
    xorg.libXcursor
    xorg.libXext
    xorg.libXi
    libxkbcommon
    vulkan-loader
    vulkan-headers
    libglvnd
    udev
    pkg-config
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.xorg.libX11
    pkgs.xorg.libXrandr
    pkgs.xorg.libXinerama
    pkgs.xorg.libXcursor
    pkgs.libxkbcommon
    pkgs.xorg.libXi
    pkgs.vulkan-loader
    pkgs.libglvnd
  ];
}
