{ pkgs ? import <nixpkgs> {} }:
with pkgs;

stdenv.mkDerivation {
  name = "time-tracker-env";
  buildInputs = [
    git
    rustChannels.stable.rust
  ];
}
