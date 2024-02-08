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

        createDbScript = ''
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

          # Starts the daemon
          ${pkgs.mariadb}/bin/mysqld --datadir=$MYSQL_DATADIR --pid-file=$MYSQL_PID_FILE \
            --socket=$MYSQL_UNIX_SOCK 2> $MYSQL_HOME/mysql.log &
          MYSQL_PID=$!

          # Wait for the server to start
          while ! mysqladmin ping &>/dev/null; do
            sleep 1
          done

          echo "MariaDB server started with PID $MYSQL_PID"

          # Create the database
          mysqladmin create dwn > /dev/null 2>&1

          export DATABASE_URL=mysql://root@localhost/dwn?unix_socket=$MYSQL_UNIX_SOCK
        '';

        trapDbScript = ''
          finish()
          {
            echo "Shutting down MariaDB server"
            mysqladmin shutdown
            pkill $MYSQL_PID
            wait $MYSQL_PID
          }

          trap finish EXIT
        '';

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
            mariadb
            nodePackages.prettier
            pkg-config
            sqlx-cli
          ];

          DATABASE_URL =
            "mysql://root@localhost/dwn?unix_socket=${pkgs.mariadb}/mysql.sock";
          SQLX_OFFLINE = true;
        };

        commonShell = {
          checks = self.checks.${localSystem};
          packages = with pkgs; [ cargo-watch rust-analyzer ];

          DATABASE_URL =
            "mysql://root@localhost/dwn?unix_socket=${pkgs.mariadb}/mysql.sock";
          SQLX_OFFLINE = true;
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

        dwn-server = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "dwn-server";
        });
      in {
        checks = { inherit dwn dwn-server cargoClippy cargoDoc; };

        apps = rec {
          migrate = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "migrate" ''
              ${pkgs.sqlx-cli}/bin/sqlx migrate run --source dwn-server/migrations
            '';
          };

          prepare = flake-utils.lib.mkApp {
            drv = pkgs.writeScriptBin "prepare" ''
              nix develop -c cargo sqlx prepare --workspace -- --all-features --all-targets --tests
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
          lib = dwn;
          server = dwn-server;

          default = pkgs.symlinkJoin {
            name = "all";
            paths = [ lib server ];
          };
        };

        devShells = {
          default = craneLib.devShell commonShell;

          db = craneLib.devShell (commonShell // {
            shellHook = ''
              ${createDbScript}
              ${trapDbScript}
            '';
          });
        };
      });
}
