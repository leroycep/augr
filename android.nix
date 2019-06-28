let native-pkgs = import <nixpkgs> {};
in
let pkgs = import <nixpkgs> {
  crossSystem = native-pkgs.lib.systems.examples.armv7a-android-prebuilt;
};
  myrust = (native-pkgs.rustChannels.stable.rust.override {
    extensions = [ "rust-std" ];
    targets = [
        "armv7-linux-androideabi"
    ];
  });
in
  pkgs.callPackage (
    {mkShell, cargo}:
    mkShell {
      nativeBuildInputs = [ myrust ];
    }
  ) {}

