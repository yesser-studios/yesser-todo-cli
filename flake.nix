{
  description = "yesser-todo-cli — A to-do CLI written in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk' = pkgs.callPackage naersk { };
        mkPackage = name: bin-name: naersk'.buildPackage {
            src = ./.;
            pname = name;
            cargoBuildOptions = x: x ++ ["-p" name];
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl ];
            meta.mainProgram = bin-name;
        };
      in {
        packages = {
          cli = mkPackage "yesser-todo-cli" "todo";
          server = mkPackage "yesser-todo-server" "yesser-todo-server";
          default = self.packages.${system}.cli;
        };
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            cargo
            rustc
            clippy
            rustfmt
            rust-analyzer
          ];

          buildInputs = with pkgs; [
            openssl
          ];
        };
      });
}
