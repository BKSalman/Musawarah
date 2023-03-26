{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
  shellHook = ''
    export $(cat .env)
  '';
}