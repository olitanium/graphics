{
  description = "Oliver's Rust Flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = input@{ self, nixpkgs, ... }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};

    buildInputs = with pkgs; [
      input.fenix.packages.${system}.complete.toolchain

      assimp
      openssl
      pkg-config

      glfw
      #glfw-wayland
      cmake
      libx11
      libxrandr
      libxinerama
      libxcursor
      libxi
      libglvnd

      zlib

      ninja
      clang
      bintools
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = buildInputs ++ [pkgs.bashInteractive];

      # shellHook = "echo Success"

      # Set Environment Variables
      LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
      LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
      RUST_BACKTRACE = "full";
    };
  };
}
