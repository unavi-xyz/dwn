{
  inputs = {
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      localSystem:
      let
        pkgs = import nixpkgs {
          inherit localSystem;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonArgs = {
          src = craneLib.cleanCargoSource (craneLib.path ./.);

          strictDeps = true;

          buildInputs =
            with pkgs;
            [ openssl ]
            ++ lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.Security
              libiconv
            ];

          nativeBuildInputs = with pkgs; [
            clang
            cmake
            pkg-config
            rustPlatform.bindgenHook
          ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        commonShell = {
          checks = self.checks.${localSystem};
          packages = with pkgs; [
            cargo-deny
            cargo-edit
            cargo-machete
            cargo-rdme
            cargo-release
            cargo-workspaces
            minio
            minio-client
            rust-analyzer
          ];

          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // { pname = "deps"; });

        cargoClippy = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "clippy";
          }
        );

        cargoDoc = craneLib.cargoDoc (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "doc";
          }
        );

        dwn-server = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            pname = "dwn-server";
            cargoExtraArgs = "-p dwn-server";
          }
        );

        treefmtEval = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
      in
      {
        formatter = treefmtEval.config.build.wrapper;

        checks = { inherit dwn-server cargoClippy cargoDoc; };

        apps = rec {
          dwn-server = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "dwn-server" ''
              ${self.packages.${localSystem}.dwn-server}/bin/dwn-server
            '';
          };

          default = dwn-server;
        };

        packages = {
          inherit dwn-server;
          default = dwn-server;
        };

        devShells = {
          default = craneLib.devShell commonShell;

          backend = craneLib.devShell (
            commonShell
            // {
              shellHook = ''
                # Start minio
                mkdir -p $PWD/minio
                ${pkgs.minio}/bin/minio server $PWD/minio > $PWD/minio/minio.log 2>&1 &
                MINIO_PID=$!

                # Wait for server to start
                while ! curl -s http://localhost:9000/minio/health/live > /dev/null; do
                  if [ $count -eq 10 ]; then
                    echo "Failed to start Minio server"
                    exit 1
                  fi
                  count=$((count+1))
                  sleep 1
                done

                echo "Minio server started with PID $MINIO_PID"

                # Create bucket
                mc alias set minio http://localhost:9000 minioadmin minioadmin 
                mc mb minio/dwn > /dev/null 2>&1

                finish()
                {
                  echo "Shutting down Minio server, PID $MINIO_PID"
                  kill -9 $MINIO_PID
                  wait $MINIO_PID
                }

                trap finish EXIT

                $SHELL
              '';

              S3_ACCESS_KEY_ID = "minioadmin";
              S3_BUCKET_NAME = "dwn";
              S3_ENDPOINT = "http://localhost:9000";
              S3_REGION = "us-east-1";
              S3_SECRET_ACCESS_KEY = "minioadmin";
            }
          );
        };
      }
    );
}
