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
              pkgsCross.windows.pthreads
            ];
            rustFlags = pkgsCross: [
              "-C" "link-arg=-L${pkgsCross.windows.pthreads}/lib"
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
            wayland-protocols
            glib
            atk
            gtk3
            gtk4
            
            # Tray icon dependencies
            libappindicator
            libappindicator-gtk3
            libayatana-appindicator
            libnotify
            
            # Wayland tray support
            waybar
            wl-clipboard
            
            # X11 compatibility for tray
            xorg.libX11
            xorg.libXtst
            
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
            echo "🖥️ Desktop: $XDG_CURRENT_DESKTOP"
            echo "🌊 Wayland: $(if [ -n "$WAYLAND_DISPLAY" ]; then echo "✓"; else echo "✗"; fi)"
            echo "🔲 X11: $(if [ -n "$DISPLAY" ]; then echo "✓"; else echo "✗"; fi)"
            echo ""
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
            echo ""
            echo "🔧 Wayland Tray Support:"
            echo "  If tray icon is not visible in Wayland, try:"
            echo "    export GDK_BACKEND=x11"
            echo "    export QT_QPA_PLATFORM=xcb"
            echo "    cargo run"
            echo ""
            echo "  Or install tray extensions for your desktop environment:"
            echo "    GNOME: gnome-shell-extension-appindicator"
            echo "    KDE: Usually works out of the box"
            echo "    Sway: Configure waybar with tray module"
            echo ""
            
            # Set environment variables for better tray support
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.libappindicator-gtk3}/lib/pkgconfig"
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.libayatana-appindicator}/lib/pkgconfig"
            
            # Ensure XDG_CURRENT_DESKTOP is set for tray support
            if [ -z "$XDG_CURRENT_DESKTOP" ]; then
              if [ -n "$WAYLAND_DISPLAY" ]; then
                export XDG_CURRENT_DESKTOP="wayland"
                echo "Set XDG_CURRENT_DESKTOP=wayland"
              elif [ -n "$DISPLAY" ]; then
                export XDG_CURRENT_DESKTOP="X11"
                echo "Set XDG_CURRENT_DESKTOP=X11"
              fi
            fi
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
              wayland-protocols
              glib
              atk
              gtk3
              gtk4
              libappindicator
              libappindicator-gtk3
              libayatana-appindicator
              libnotify
              wl-clipboard
              xorg.libX11
              xorg.libXtst
            ];
            
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            
            # Environment variables for tray support
            preBuild = ''
              export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.libappindicator-gtk3}/lib/pkgconfig"
              export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgs.libayatana-appindicator}/lib/pkgconfig"
            '';
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