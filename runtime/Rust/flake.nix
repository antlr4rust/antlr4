{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix.url = "github:nix-community/fenix";
  };

  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
    fenix,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [fenix.overlays.default];
        };

        toolchain = pkgs.fenix.complete.toolchain;
        
        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };

        dependencies = with pkgs; [
            jdk25
            maven
            perl
            pkg-config
            openssl
        ];

      in {
        defaultPackage = naersk'.buildPackage {
          src = ./.;
          nativeBuildInputs = dependencies;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = dependencies ++ [ toolchain ];
        };
      }
    );
}
