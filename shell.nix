{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  NIX_SHELL = "Homage";

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

  packages = with pkgs; [
    cargo
    clippy
    pkg-config
    rust-analyzer
    rustc
    rustfmt
  ];
}
