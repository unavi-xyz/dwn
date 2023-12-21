{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, flake-utils, nixpkgs, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustBin = pkgs.rust-bin.stable.latest.default;

        build_inputs = with pkgs; [ ];

        native_build_inputs = with pkgs; [ cargo-auditable openssl pkg-config ];

        code = pkgs.callPackage ./. {
          inherit pkgs system build_inputs native_build_inputs;
        };
      in rec {
        packages = code // {
          all = pkgs.symlinkJoin {
            name = "all";
            paths = with code; [ dwn-server dwn ];
          };

          default = packages.all;
          override = packages.all;
          overrideDerivation = packages.all;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs;
            [ cargo-watch rust-analyzer rustBin mariadb ] ++ build_inputs;
          nativeBuildInputs = native_build_inputs;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath build_inputs;

          shellHook = ''
            MYSQL_BASEDIR=${pkgs.mariadb}
            MYSQL_HOME=$PWD/mysql
            MYSQL_DATADIR=$MYSQL_HOME/data
            export MYSQL_UNIX_PORT=$MYSQL_HOME/mysql.sock
            MYSQL_PID_FILE=$MYSQL_HOME/mysql.pid
            alias mysql='mysql -u root'

            if [ ! -d "$MYSQL_HOME" ]; then
              # Make sure to use normal authentication method otherwise we can only
              # connect with unix account. But users do not actually exists in nix.
              mysql_install_db --auth-root-authentication-method=normal \
                --datadir=$MYSQL_DATADIR --basedir=$MYSQL_BASEDIR \
                --pid-file=$MYSQL_PID_FILE
            fi

            # Starts the daemon
            mysqld --datadir=$MYSQL_DATADIR --pid-file=$MYSQL_PID_FILE \
              --socket=$MYSQL_UNIX_PORT 2> $MYSQL_HOME/mysql.log &
            MYSQL_PID=$!

            finish()
            {
              mysqladmin -u root --socket=$MYSQL_UNIX_PORT shutdown
              kill $MYSQL_PID
              wait $MYSQL_PID
            }
            trap finish EXIT

            # Initialize the database
            sh ./sql/run.sh
          '';
        };
      });
}
