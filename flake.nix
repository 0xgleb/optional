{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";

    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";

    devenv.url = "github:cachix/devenv/v1.7";
    devenv.inputs = {
      nixpkgs.follows = "nixpkgs";
      git-hooks.follows = "git-hooks";
    };

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, flake-utils, devenv, git-hooks, fenix, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };
        toolchain = fenix.packages.${system}.default;

        cargo-stylus = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-stylus";
          version = "0.6.3";
          cargoHash = "sha256-BuGTg2VW4xQufWedFTVdefJAg4LFn2vpd0/5rdCGss0=";

          src = pkgs.fetchFromGitHub {
            owner = "OffchainLabs";
            repo = "cargo-stylus";
            rev = "v${version}";
            hash = "sha256-iaKTcc0LEwrTwLOwwCwXzFIB1LjRC9Tt2ljklE4ujPg=";
          };

          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs;
            [ openssl ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.libiconv
            ];

          OPENSSL_NO_VENDOR = 1;
          doCheck = false;
        };

        env = { };
        src = ./.;
        hooks = {
          # Nix
          nil.enable = true;
          nixfmt-classic.enable = true;
          deadnix.enable = true;
          statix.enable = true;

          # Rust
          taplo.enable = true;
          rustfmt.enable = true;
          rustfmt.packageOverrides = { inherit (toolchain) cargo rustfmt; };

          # TypeScript
          eslint.enable = true;
          prettier.enable = true;

          # Misc
          denofmt.enable = true;
          shellcheck.enable = true;
        };

      in {
        devShells = {
          default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [{
              # https://devenv.sh/reference/options/
              packages = with pkgs;
                [ cargo-watch cargo-stylus ] ++ lib.optionals stdenv.isDarwin
                (with darwin.apple_sdk; [
                  libiconv
                  frameworks.Security
                  frameworks.CoreFoundation
                  frameworks.SystemConfiguration
                ]);

              languages = {
                nix.enable = true;

                javascript.enable = true;
                javascript.pnpm.enable = true;
                typescript.enable = true;

                rust = {
                  enable = true;
                  inherit toolchain;
                };
              };

              inherit env;
              git-hooks = { inherit hooks; };
              difftastic.enable = true;
              cachix.enable = true;

              # Disable process-compose as we don't need it
              process.managers.process-compose.enable = false;
            }];
          };
        };

        packages = { };

        checks.git-hooks = git-hooks.lib.${system}.run { inherit hooks src; };
      });

  nixConfig = {
    extra-substituters = "https://devenv.cachix.org";
    extra-trusted-public-keys =
      "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    allow-unfree = true;
  };
}
