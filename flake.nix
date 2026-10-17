{
  description = "Schafkopf-Statistik: Rust-Kern und Android-App";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);

      # Android SDK/NDK sind unfrei lizenziert; die Lizenz wird hier ausdrücklich akzeptiert.
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
        config = {
          allowUnfree = true;
          android_sdk.accept_license = true;
        };
      };
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          buildTools = "34.0.0";
          android = pkgs.androidenv.composeAndroidPackages {
            platformVersions = [ "34" ];
            buildToolsVersions = [ buildTools ];
            includeNDK = true;
            includeEmulator = false;
            includeSystemImages = false;
          };
          sdk = "${android.androidsdk}/libexec/android-sdk";
          rustAndroid = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "clippy" "rustfmt" "rust-analyzer" ];
            targets = [ "aarch64-linux-android" "armv7-linux-androideabi" "x86_64-linux-android" ];
          };
        in
        {
          # Rust-Kern entwickeln und testen: nix develop
          default = pkgs.mkShell {
            packages = with pkgs; [ rustc cargo clippy rustfmt rust-analyzer ];
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };

          # App am Desktop ausprobieren (WebView): nix develop .#desktop
          desktop = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
              clippy
              rustfmt
              dioxus-cli
              pkg-config
              openssl
              webkitgtk_4_1
              gtk3
              libsoup_3
              xdotool
              glib
            ];
          };

          # Android-App bauen und aufs Handy bringen: nix develop .#android
          android = pkgs.mkShell {
            packages = [
              rustAndroid
              pkgs.dioxus-cli
              pkgs.jdk17
              pkgs.pkg-config
              pkgs.openssl
              android.androidsdk
            ];
            ANDROID_HOME = sdk;
            ANDROID_SDK_ROOT = sdk;
            JAVA_HOME = "${pkgs.jdk17}";
            # Gradle würde sonst ein eigenes aapt2 laden, das unter Nix nicht läuft.
            GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${sdk}/build-tools/${buildTools}/aapt2";
            shellHook = ''
              export NDK_HOME="$(echo ${sdk}/ndk/* | cut -d' ' -f1)"
              export ANDROID_NDK_HOME="$NDK_HOME"
              echo "Android SDK: $ANDROID_HOME"
              echo "Android NDK: $NDK_HOME"
            '';
          };
        });

      formatter = forAllSystems (system: (pkgsFor system).nixpkgs-fmt);
    };
}
