{
  description = "image viewing of the past";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
  }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
      packages = with pkgs; [
        libGL

        wayland
        libxkbcommon
      ];
      name = "imvr";

      version = "0.1.1";
    in {
      # Build the package
      packages = rec {
        fish = pkgs.rustPlatform.buildRustPackage {
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "ext-0.1.0" = "sha256-50llOwQPEBNmEkDV6quVyNOkZFa78IV0a+eoxHqvVPA=";
            };
          };

          pname = name;
          inherit version;

          src = ./.;

          # doCheck = false;

          nativeBuildInputs = with pkgs; [pkg-config cargo];
          buildInputs = packages;

          installPhase = let
            target = "target/${pkgs.stdenv.targetPlatform.config}/release";
          in ''
            install -Dm755 ${target}/${name} $out/bin/${name}
          '';

          postFixup = ''
            makeWrapper $out/bin/${name} --suffix LD_LIBRARY_PATH : ${pkgs.wayland}/lib/libwayland-client.so
          '';

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
        };
        default = fish;
      };
    });
}
