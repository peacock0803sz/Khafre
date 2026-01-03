{
  description = "Khafre - Sphinx documentation Editor with live preview";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, self', inputs', pkgs, system, ... }: {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Rust
            cargo
            rustc
            rust-analyzer
            rustfmt
            clippy

            # Dioxus CLI
            dioxus-cli

            # System dependencies
            pkg-config
          ] ++ lib.optionals stdenv.hostPlatform.isLinux [
            # Linux-specific dependencies for Dioxus desktop
            gtk3
            webkitgtk
            libsoup_3
            glib
            # Wayland dependencies
            wayland
            libxkbcommon
          ] ++ lib.optionals stdenv.hostPlatform.isDarwin [
            # macOS-specific - use libiconv for builds
            libiconv
          ];

          # Set up library paths for Linux
          shellHook = pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
              pkgs.gtk3
              pkgs.webkitgtk
              pkgs.libsoup_3
              pkgs.glib
              pkgs.wayland
              pkgs.libxkbcommon
            ]}:$LD_LIBRARY_PATH"
          '';
        };
      };
    };
}
