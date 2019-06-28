let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  native-pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  pkgs = import <nixpkgs> { crossSystem = native-pkgs.lib.systems.examples.armv7a-android-prebuilt; };
  rust = (native-pkgs.rustChannels.stable.rust.override {
    targets = [
        "armv7-linux-androideabi"
    ];
  });
in
  pkgs.callPackage (
    {mkShell, cargo}:
    mkShell {
      nativeBuildInputs = [ rust ];
    }
  ) {}

