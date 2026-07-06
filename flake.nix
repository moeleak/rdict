{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        };
        windowsPkgs = pkgs.pkgsCross.mingwW64;
        nativePackages = pkgs.callPackage ./package.nix { };
        windowsPackages = windowsPkgs.callPackage ./package.nix { };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            rust-analyzer
            rustc
            rustfmt
          ];
        };

        packages = {
          inherit (nativePackages) default rdict rdict-telegram rdict-iced rdict-iced-android;
          rdict-iced-windows = windowsPackages.rdict-iced;
        };
      }
    );
}
