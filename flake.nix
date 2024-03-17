{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (localSystem:
      let
        pkgs = import nixpkgs {
          inherit localSystem;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable.latest.default;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonArgs = {
          src = lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (lib.hasSuffix ".sql" path) || (lib.hasInfix "/.sqlx/" path)
              || (craneLib.filterCargoSources path type);
          };

          strictDeps = true;

          buildInputs = with pkgs;
            [ openssl ] ++ lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.libiconv
            ];

          nativeBuildInputs = with pkgs; [
            cargo-auditable
            clang
            mariadb
            minio
            minio-client
            nodePackages.prettier
            pkg-config
            sqlx-cli
          ];

          DATABASE_URL = "mysql://root@localhost/dwn";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          SQLX_OFFLINE = true;
        };

        commonShell = {
          checks = self.checks.${localSystem};
          packages = with pkgs; [ cargo-rdme cargo-watch rust-analyzer ];

          DATABASE_URL = "mysql://root@localhost/dwn";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        cargoArtifacts =
          craneLib.buildDepsOnly (commonArgs // { pname = "deps"; });

        cargoClippy = craneLib.cargoClippy (commonArgs // {
          inherit cargoArtifacts;
          pname = "clippy";
        });

        cargoDoc = craneLib.cargoDoc (commonArgs // {
          inherit cargoArtifacts;
          pname = "doc";
        });

        dwn = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "dwn";
        });
      in {
        checks = { inherit dwn cargoClippy cargoDoc; };

        apps = rec {
          reset = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "reset" ''
              ${pkgs.sqlx-cli}/bin/sqlx database reset -y
            '';
          };

          dwn = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "dwn" ''
              ${self.packages.${localSystem}.dwn}/bin/dwn
            '';
          };

          generate-readme = flake-utils.lib.mkApp {
            drv = pkgs.writeShellScriptBin "generate-readme" ''
              cargo rdme
            '';
          };

          default = dwn;
        };

        packages = {
          dwn = dwn;
          default = dwn;
        };

        devShells = {
          default = craneLib.devShell commonShell;

          server = craneLib.devShell (commonShell // {
            shellHook = ''
              MYSQL_BASEDIR=${pkgs.mariadb}
              MYSQL_HOME=$PWD/mysql
              MYSQL_DATADIR=$MYSQL_HOME/data
              MYSQL_UNIX_SOCK=$MYSQL_HOME/mysql.sock
              MYSQL_PID_FILE=$MYSQL_HOME/mysql.pid

              alias mysqladmin="${pkgs.mariadb}/bin/mysqladmin -u root --socket $MYSQL_UNIX_SOCK"

              if [ ! -d "$MYSQL_HOME" ]; then
                # Make sure to use normal authentication method otherwise we can only
                # connect with unix account. But users do not actually exists in nix.
                ${pkgs.mariadb}/bin/mysql_install_db --auth-root-authentication-method=normal \
                  --datadir=$MYSQL_DATADIR --basedir=$MYSQL_BASEDIR \
                  --pid-file=$MYSQL_PID_FILE
              fi

              # Starts mariadb
              ${pkgs.mariadb}/bin/mysqld --datadir=$MYSQL_DATADIR --pid-file=$MYSQL_PID_FILE \
                --socket=$MYSQL_UNIX_SOCK 2> $MYSQL_HOME/mysql.log &
              MYSQL_PID=$!

              # Start minio
              mkdir -p $PWD/minio
              ${pkgs.minio}/bin/minio server $PWD/minio > $PWD/minio/minio.log 2>&1 &
              MINIO_PID=$!

              # Wait for servers to start
              count=0
              while ! mysqladmin ping &>/dev/null; do
                if [ $count -eq 10 ]; then
                  echo "Failed to start MariaDB server"
                  exit 1
                fi
                count=$((count+1))
                sleep 1
              done

              echo "MariaDB server started with PID $MYSQL_PID"

              while ! curl -s http://localhost:9000/minio/health/live > /dev/null; do
                if [ $count -eq 10 ]; then
                  echo "Failed to start Minio server"
                  exit 1
                fi
                count=$((count+1))
                sleep 1
              done

              echo "Minio server started with PID $MINIO_PID"

              # Create database
              mysqladmin create dwn > /dev/null 2>&1
              export DATABASE_URL=mysql://root@localhost/dwn?unix_socket=$MYSQL_UNIX_SOCK

              # Run migrations
              ${pkgs.sqlx-cli}/bin/sqlx migrate run

              # Create bucket
              mc alias set minio http://localhost:9000 minioadmin minioadmin 
              mc mb minio/dwn > /dev/null 2>&1

              finish()
              {
                echo "Shutting down MariaDB server, PID $MYSQL_PID"
                mysqladmin shutdown
                echo "Shutting down Minio server, PID $MINIO_PID"
                kill -9 $MINIO_PID
                wait $MYSQL_PID
                wait $MINIO_PID
              }

              trap finish EXIT

              $SHELL
            '';
          });
        };
      });
}
