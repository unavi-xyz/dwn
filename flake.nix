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
            nodePackages.prettier
            pkg-config
          ];
        };

        commonShell = {
          checks = self.checks.${localSystem};
          packages = with pkgs; [ cargo-watch mariadb rust-analyzer sqlx-cli ];
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

        dwn-server = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "dwn-server";
        });
      in {
        checks = { inherit dwn-server cargoClippy cargoDoc; };

        apps = rec {
          migrate = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "migrate" ''
              ${pkgs.sqlx-cli}/bin/sqlx database create
              ${pkgs.sqlx-cli}/bin/sqlx migrate run
            '';
          };

          prepare = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "prepare" ''
              ${rustToolchain}/bin/cargo sqlx prepare --workspace -- --all-targets --all-features --tests
            '';
          };

          server = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "server" ''
              ${self.packages.${localSystem}.server}/bin/dwn-server
            '';
          };

          default = server;
        };

        packages = rec {
          server = dwn-server;

          default = pkgs.symlinkJoin {
            name = "all";
            paths = [ server ];
          };
        };

        devShells = {
          default = craneLib.devShell commonShell;

          db = craneLib.devShell (commonShell // {
            shellHook = ''
              MYSQL_BASEDIR=${pkgs.mariadb}
              MYSQL_HOME=$PWD/mysql
              MYSQL_DATADIR=$MYSQL_HOME/data
              MYSQL_UNIX_PORT=$MYSQL_HOME/mysql.sock
              MYSQL_PID_FILE=$MYSQL_HOME/mysql.pid

              if [ ! -d "$MYSQL_HOME" ]; then
                # Make sure to use normal authentication method otherwise we can only
                # connect with unix account. But users do not actually exists in nix.
                mysql_install_db --auth-root-authentication-method=normal \
                  --datadir=$MYSQL_DATADIR --basedir=$MYSQL_BASEDIR \
                  --pid-file=$MYSQL_PID_FILE
              fi

              # Starts the daemon
              ${pkgs.mariadb}/bin/mysqld --datadir=$MYSQL_DATADIR --pid-file=$MYSQL_PID_FILE \
                --socket=$MYSQL_UNIX_PORT 2> $MYSQL_HOME/mysql.log &
              MYSQL_PID=$!

              # Wait for the server to start
              while ! ${pkgs.mariadb}/bin/mysqladmin -u root --socket=$MYSQL_UNIX_PORT ping &>/dev/null; do
                sleep 1
              done

              echo "MariaDB server started with PID $MYSQL_PID"

              # Create the database
              ${pkgs.mariadb}/bin/mysqladmin -u root --socket $MYSQL_UNIX_PORT create dwn || true

              finish()
              {
                echo "Shutting down MariaDB server"
                ${pkgs.mariadb}/bin/mysqladmin -u root --socket=$MYSQL_UNIX_PORT shutdown
                pkill $MYSQL_PID
                wait $MYSQL_PID
              }

              trap finish EXIT
            '';
          });
        };
      });
}
