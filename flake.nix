{
  description = "Clipboard QR Code Application";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Define build targets similar to the reference
        buildTargets = {
          "x86_64-linux" = {
            nixPkgsSystem = "x86_64-unknown-linux-gnu";
            rustTarget = "x86_64-unknown-linux-gnu";
          };
          "x86_64-windows" = {
            nixPkgsSystem = "x86_64-w64-mingw32";
            rustTarget = "x86_64-pc-windows-gnu";
            nativeBuildInputs = pkgsCross: [
              pkgsCross.stdenv.cc
              # pkgsCross.windows.pthreads
            ];
            rustFlags = pkgsCross: [
              # "-C" "link-arg=-L${pkgsCross.windows.pthreads}/lib"
            ];
          };
        };

        # Helper function to create cross-compilation environment
        mkCrossPkgs = targetSystem: import nixpkgs {
          inherit system overlays;
          crossSystem = {
            config = buildTargets.${targetSystem}.nixPkgsSystem;
          };
        };

        # Helper function to create build environment
        mkBuildEnv = targetSystem: let
          buildTarget = buildTargets.${targetSystem};
          pkgsCross = mkCrossPkgs targetSystem;
        in {
          nativeBuildInputs = (buildTarget.nativeBuildInputs or (pkgsCross: [])) pkgsCross ++ [
            (pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = [ buildTarget.rustTarget ];
            })
          ];

          # Environment variables for cross-compilation
          TARGET_CC = "${pkgsCross.stdenv.cc}/bin/${pkgsCross.stdenv.cc.targetPrefix}cc";
          CARGO_BUILD_TARGET = buildTarget.rustTarget;
          CARGO_BUILD_RUSTFLAGS = [
            "-C" "linker=${pkgsCross.stdenv.cc}/bin/${pkgsCross.stdenv.cc.targetPrefix}cc"
          ] ++ (buildTarget.rustFlags or (pkgsCross: [])) pkgsCross;
        };

      in {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Build tools
            pkg-config
            
            # System libraries
            openssl
            libxkbcommon
            wayland
            glib
            atk
            gtk3
            
            # Tray icon dependencies
            libappindicator
            libayatana-appindicator
            libnotify
            
            # Development tools
            cargo-watch
            cargo-edit
            cargo-audit
            cargo-tarpaulin
            
            # Code formatting and linting
            rustfmt
            clippy
            
            # Documentation
            mdbook

            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
              targets = [ "x86_64-pc-windows-gnu" "x86_64-unknown-linux-gnu" ];
            })
          ];
          
          shellHook = ''
            echo "🚀 Clipboard QR Development Environment"
            echo "Available commands:"
            echo "  cargo build          - Build for current platform"
            echo "  cargo build --release - Build release for current platform"
            echo "  cargo build --target x86_64-pc-windows-gnu --release - Cross-compile for Windows"
            echo "  cargo test           - Run tests"
            echo "  cargo clippy         - Run linter"
            echo "  cargo fmt            - Format code"
            echo "  cargo watch -x run   - Watch and run"
            echo ""
            echo "Cross-compilation targets:"
            echo "  x86_64-linux:   cargo build --target x86_64-unknown-linux-gnu --release"
            echo "  x86_64-windows: cargo build --target x86_64-pc-windows-gnu --release"
          '';
        };

        # Cross-compilation development shells
        devShells = {
          # Windows cross-compilation shell
          windows = pkgs.mkShell (mkBuildEnv "x86_64-windows");
        };

        packages = {
          # Build for current platform
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "clipboard-qr";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            
            buildInputs = with pkgs; [
              pkg-config
              openssl
              libxkbcommon
              wayland
              glib
              atk
              gtk3
              libappindicator
              libayatana-appindicator
              libnotify
            ];
            
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
          };

          windows = let
            env = mkBuildEnv "x86_64-windows";
            pkgsCross = mkCrossPkgs "x86_64-windows";
          in pkgsCross.rustPlatform.buildRustPackage {
            pname = "clipboard-qr";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            
            buildInputs = with pkgsCross; [
              windows.pthreads
            ];
            
            nativeBuildInputs = env.nativeBuildInputs;
            
            # Set environment variables
            inherit (env) TARGET_CC CARGO_BUILD_TARGET CARGO_BUILD_RUSTFLAGS;
          };
        };

        apps = {
          default = {
            type = "app";
            program = "${self.packages.${system}.default}/bin/clipboard-qr";
          };
        };
      }
    );
} 