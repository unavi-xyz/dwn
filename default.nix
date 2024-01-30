{ lib, pkgs, system, build_inputs, native_build_inputs, makeRustPlatform }:
let
  rustBin = pkgs.rust-bin.stable.latest.default;

  rustPlatform = makeRustPlatform {
    cargo = rustBin;
    rustc = rustBin;
  };

  common = {
    version = "0.0.0";
    src = ./.;
    cargoLock.lockFile = ./Cargo.lock;
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

    buildInputs = build_inputs;
    nativeBuildInputs = native_build_inputs;

    LD_LIBRARY_PATH = lib.makeLibraryPath build_inputs;
  };

  SQLX_OFFLINE = true;
in {
  dwn-server = rustPlatform.buildRustPackage (common // {
    pname = "dwn-server";
    buildAndTestSubdir = "dwn-server";
  });
  dwn = rustPlatform.buildRustPackage (common // {
    pname = "dwn";
    buildAndTestSubdir = "dwn";
  });
}
