{
  description = "github-insight";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane/v0.17.3";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
      crane,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Get Rust toolchain from fenix - with updated hash
        rust-toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
        };

        # Create a crane lib with our rust-toolchain
        craneLib = (crane.mkLib pkgs).overrideToolchain rust-toolchain;

        # Create a modified buildRustPackage that skips the problematic steps
        buildRustPackageCustom =
          args:
          pkgs.rustPlatform.buildRustPackage (
            args
            // {
              # These options help bypass the workspace inheritance issues
              dontFixCargo = true;
              cargoLockCheck = false;
              doCheck = false;
            }
          );

        # Build cargo-machete with the same rust-toolchain
        cargo-machete = craneLib.buildPackage {
          pname = "cargo-machete";
          version = "0.8.0";

          src = pkgs.fetchCrate {
            pname = "cargo-machete";
            version = "0.8.0";
            sha256 = "sha256-EMU/ZegrNBzDtjifdVlHP/P9hNJJ//SDDwlB7uo1sY0=";
          };

          # Ensure the binary is installed in the expected location
          cargoExtraArgs = "--bin cargo-machete";

          doCheck = false;
        };
      in
      {
        # Development shell with Rust toolchain
        devShells.default = craneLib.devShell {
          packages = [
            pkgs.nixpkgs-fmt
            pkgs.openssl
            pkgs.pkg-config
            pkgs.nodejs
            pkgs.nodePackages.npm
            pkgs.go-task
            cargo-machete
            pkgs.protobuf
            pkgs.ntfy-sh
            pkgs.chromium
          ];

          inputsFrom = [ ];

          # Add OpenSSL configuration
          shellHook = ''
            export OPENSSL_DIR=${pkgs.openssl.dev}
            export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
            export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
            echo "Shell loaded successfully with OpenSSL configuration"
          '';
        };

        # Package that builds from source
        packages.default = buildRustPackageCustom {
          pname = "github-insight";
          version = "0.1.3";
          src = ./.;

          # Basic cargo lock configuration
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          # Enable Git fetching with CLI
          CARGO_NET_GIT_FETCH_WITH_CLI = "true";
          CARGO_TERM_VERBOSE = "true";

          nativeBuildInputs = [
            rust-toolchain
            pkgs.pkg-config
            pkgs.protobuf
          ];

          buildInputs =
            [
              pkgs.openssl
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];

          # OpenSSL environment variables
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
        };

        # Binary package for GitHub releases
        packages.github-insight-binary = pkgs.stdenv.mkDerivation rec {
          pname = "github-insight";
          version = "0.1.3";

          src = pkgs.fetchurl {
            url = "https://github.com/tacogips/github-insight/releases/download/v${version}/github-insight-${pkgs.stdenv.hostPlatform.system}.tar.gz";
            sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; # Update this hash for each release
          };

          dontBuild = true;
          dontConfigure = true;

          installPhase = ''
            mkdir -p $out/bin
            cp github-insight-mcp $out/bin/
            cp github-insight-cli $out/bin/
            chmod +x $out/bin/*
          '';
        };
      }
    );
}
