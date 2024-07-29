{
  inputs = {
    nixpkgs.url = "github:alexandru0-dev/nixpkgs/fix/webkitgtk_6_0";
    # nixpkgs.url = "github:alexandru0-dev/nixpkgs/master";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          nativeBuildInputs = with pkgs; [ 
            pkg-config
            libsoup_3.dev
          ];
        in
        with pkgs;
        {
          devShells.default = mkShell {
            inherit nativeBuildInputs;
            buildInputs = with pkgs; [ 
              rust-bin.stable.latest.default
              gdk-pixbuf.dev
              webkitgtk_6_0.dev
              webkitgtk.dev
              gtk4.dev
              barlow
            ];
          };
        }
      );
}

