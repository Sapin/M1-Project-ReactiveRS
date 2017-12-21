{ nixpkgs ? import <nixpkgs> {} }:
let
  inherit (nixpkgs) pkgs;
  packages = with nixpkgs.rustChannels.stable; [ rust ];
in pkgs.stdenv.mkDerivation {
  name = "programmation-parallele-tp1";
  buildInputs = packages ++ [ pkgs.SDL2 ];
}

