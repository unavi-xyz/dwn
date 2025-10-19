_: {
  perSystem =
    { pkgs, lib, ... }:
    let
      pname = "dwn-server";

      src = lib.fileset.toSource rec {
        root = ../..;
        fileset = lib.fileset.unions [
          (pkgs.crane.fileset.commonCargoSources root)
          ../../LICENSE-APACHE
          ../../LICENSE-MIT
          ./README.md
        ];
      };

      cargoArgs = rec {
        inherit pname;
        inherit src;

        cargoExtraArgs = "-p ${pname}";
        strictDeps = true;

        runtimeDependencies = [ ];

        nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [ pkg-config ]);

        buildInputs = runtimeDependencies;

        LD_LIBRARY_PATH = lib.makeLibraryPath runtimeDependencies;
      };

      cargoArtifacts = pkgs.crane.buildDepsOnly cargoArgs;
    in
    {
      checks = {
        "${pname}-doc" = pkgs.crane.cargoDoc (cargoArgs // { inherit cargoArtifacts; });
        "${pname}-doctest" = pkgs.crane.cargoDocTest (cargoArgs // { inherit cargoArtifacts; });
        "${pname}-nextest" = pkgs.crane.cargoNextest (
          cargoArgs
          // {
            inherit cargoArtifacts;
            cargoExtraArgs = cargoArgs.cargoExtraArgs + " --no-tests pass";
          }
        );
      };

      packages."${pname}" = pkgs.crane.buildPackage (
        cargoArgs
        // {
          inherit cargoArtifacts;
          doCheck = false;

          postInstall = ''
            mv $out/bin/* $out
            rm -r $out/bin
            cp crates/dwn-server/README.md $out
            cp LICENSE-APACHE $out
            cp LICENSE-MIT $out
          '';
        }
      );
    };
}
