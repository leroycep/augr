{ pkgs ? import <nixpkgs> {} }:
with pkgs;

let
  myrust = (rustChannels.stable.rust.override {
    extensions = [ "rust-std" ];
    targets = [
        "arm-linux-androideabi"
        "aarch64-linux-android"
        # "i686-linux-android"
        # "x86_64-linux-android"
    ];
  });
in
  stdenv.mkDerivation {
    name = "taskapp-env";
    buildInputs = [
      git
      myrust
      pkgconfig
      openssl
      gcc
      libstdcxx5
      android-studio
      rust-bindgen
    ];
  }
