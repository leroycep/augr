{ pkgs ? import <nixpkgs> {} }:
with pkgs;

stdenv.mkDerivation {
  name = "augr-env";
  buildInputs = [
    git
    rustChannels.stable.rust
  ];
}
