let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  native-pkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  pkgs = import <nixpkgs> { crossSystem = native-pkgs.lib.systems.examples.armv7a-android-prebuilt; };
  rust = native-pkgs.rustChannelOfTargets "stable" null [
    "armv7-linux-androideabi"
  ];
in
  pkgs.callPackage (
    {mkShell}:
    mkShell {
      nativeBuildInputs = [ rust native-pkgs.gcc ];
    }
  ) {}

