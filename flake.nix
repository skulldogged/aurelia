{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        inherit (nixpkgs) lib;

        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system;
          inherit overlays;
          config.allowUnfree = true;
          config.android_sdk.accept_license = true;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src"];
          targets = [
            "aarch64-apple-ios"
            "aarch64-apple-ios-sim"
            "aarch64-linux-android"
            "armv7-linux-androideabi"
            "i686-linux-android"
            "x86_64-linux-android"
          ];
        };

        androidComposition = pkgs.androidenv.composeAndroidPackages {
          cmdLineToolsVersion = "13.0";
          platformToolsVersion = "36.0.1";
          buildToolsVersions = ["36.0.0"];
          platformVersions = ["36"];
          abiVersions = ["arm64-v8a" "x86_64"];
          includeNDK = true;
          ndkVersions = ["29.0.14206865"];
        };

        androidSdk = androidComposition.androidsdk;
      in {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            pkg-config
            gobject-introspection
            cargo-tauri
            rustToolchain
            bun
            wrapGAppsHook4
            jdk17
            cargo-ndk
            android-studio
          ];

          buildInputs = with pkgs;
            lib.optionals stdenv.isLinux [
              alsa-lib
              at-spi2-atk
              atkmm
              bitwarden-cli
              cairo
              gdk-pixbuf
              glib
              gtk3
              harfbuzz
              librsvg
              libsoup_3
              pango
              webkitgtk_4_1
              openssl
              libayatana-appindicator
            ];

          ANDROID_HOME = "${androidSdk}/libexec/android-sdk";
          NDK_HOME = "${androidSdk}/libexec/android-sdk/ndk/29.0.14206865";
          JAVA_HOME = pkgs.jdk17.home;
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
          __NV_DISABLE_EXPLICIT_SYNC = 1;
          shellHook = ''
            export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
            export PATH="${androidSdk}/libexec/android-sdk/platform-tools:$PATH"
            ${lib.optionalString pkgs.stdenv.isDarwin "unset SDKROOT"}
          '';
        };
      }
    );
}
