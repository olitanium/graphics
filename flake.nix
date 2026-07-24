{
  description = "Oliver's Rust Flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    fenix.url = "github:nix-community/fenix/monthly";
  };

  outputs = input@{ self, nixpkgs, ... }: let
    system = "x86_64-linux";
    pkgs = input.nixpkgs.legacyPackages.${system};
    
    rust_toolchain = input.fenix.packages.${system}.complete.toolchain;

    buildInputs = with pkgs; [
      rust_toolchain

      glfw
      cmake
      
      libx11
      libxrandr
      libxinerama
      libxcursor
      libxi
      
      libglvnd


      
      assimp
      clang
      pkg-config
      zlib
      openssl
      bintools
    ];
  in {
    devShells.${system}.default = pkgs.mkShellNoCC {
      buildInputs = buildInputs ++ [pkgs.bashInteractive];

      #shellHook = ''
      #  cargo clean &&
      #  cargo run -r;
      #'';

      # Set Environment Variables
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
      LIBCLANG_PATH = pkgs.lib.makeLibraryPath [pkgs.libclang];
    
      RUST_BACKTRACE = "full";
    };
  };
}
