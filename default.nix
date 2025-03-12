{ pkgs ? import <nixpkgs> { } }:
let 
  pkg-config = pkgs.pkg-config;
  lib = pkgs.lib;
  stdenv = pkgs.stdenv;
in 
pkgs.rustPlatform.buildRustPackage rec {
  pname = "flix-launcher";
  version = "0.1";

  cargoLock.lockFile = ./Cargo.lock;

  src = pkgs.lib.cleanSource ./.;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = with pkgs; [
    libgit2
    openssl
    expat
    fontconfig
    libGL
    libxkbcommon
    wayland
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    xorg.libxcb
  ];

  postFixup = lib.optionalString stdenv.hostPlatform.isLinux ''
    patchelf $out/bin/flix-launcher \
      --add-rpath ${
        lib.makeLibraryPath [
          pkgs.fontconfig
          pkgs.libGL
          pkgs.libxkbcommon
          pkgs.wayland
        ]
      }
  '';

  env = {
    LIBGIT2_NO_VENDOR = 1;
  };
}
