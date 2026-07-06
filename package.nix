{
  lib,
  stdenv,
  buildPackages,
  fetchurl,
  rustPlatform,
  rustc,
  installShellFiles,
  libiconv,
  writableTmpDirAsHomeHook,
  androidenv,
  cargo-apk,
  jdk_headless,
}:
let
  version = "0.3.0";
  src = lib.cleanSource ./.;
  cargoHash = "sha256-5jkmRbp2WawfAvgCIj4L3XWYIFfnvfcsNrNfL09P344=";
  androidTarget = "aarch64-linux-android";
  androidSdkPackages = androidenv.composeAndroidPackages {
    platformVersions = [ "35" ];
    buildToolsVersions = [ "35.0.0" ];
    includeNDK = true;
    ndkVersions = [ "27.2.12479018" ];
  };
  androidSdk = androidSdkPackages.androidsdk;
  hostRustTarget = stdenv.hostPlatform.rust.rustcTarget;
  hostRustHash =
    {
      aarch64-apple-darwin = "sha256-A4KsDnrKsNqiRxBwFELp/ShBSLdSZb2y9Z44tgsDrDw=";
      x86_64-apple-darwin = "sha256-mYr3UP4sKPDb8ZPtxJbJp0B4UEhpaPQKwRqMit+3z6Y=";
      x86_64-unknown-linux-gnu = "sha256-LgM48Y7LqkoPYxuegOi44mu2/nfdVFT7qKcM+WwehKE=";
      aarch64-unknown-linux-gnu = "sha256-CUycNlMZEcXMfdarLTBpq43NdE1iObC9oTh7JD38OR4=";
    }
    .${hostRustTarget} or (throw "Unsupported Rust host target for Android build: ${hostRustTarget}");
  hostRust = fetchurl {
    url = "https://static.rust-lang.org/dist/rust-${rustc.version}-${hostRustTarget}.tar.xz";
    hash = hostRustHash;
  };
  androidRustStd = fetchurl {
    url = "https://static.rust-lang.org/dist/rust-std-${rustc.version}-${androidTarget}.tar.xz";
    hash = "sha256-3l6PpdlVgJiR7qd2goEfyQvnBfeIg72UBx6Y9ac40Fs=";
  };
  androidRustToolchain = stdenv.mkDerivation {
    pname = "rust-with-${androidTarget}-std";
    inherit (rustc) version;

    src = hostRust;
    dontUnpack = true;
    dontStrip = true;

    installPhase = ''
      runHook preInstall

      toolchain_unpack="$TMPDIR/rust-toolchain"
      android_std_unpack="$TMPDIR/rust-android-std"
      mkdir -p "$toolchain_unpack" "$android_std_unpack"

      tar -xJf ${hostRust} -C "$toolchain_unpack"
      patchShebangs "$toolchain_unpack/rust-${rustc.version}-${hostRustTarget}/install.sh"
      "$toolchain_unpack/rust-${rustc.version}-${hostRustTarget}/install.sh" \
        --prefix="$out" \
        --components=rustc,cargo,rust-std-${hostRustTarget}

      tar -xJf ${androidRustStd} -C "$android_std_unpack"
      patchShebangs "$android_std_unpack/rust-std-${rustc.version}-${androidTarget}/install.sh"
      "$android_std_unpack/rust-std-${rustc.version}-${androidTarget}/install.sh" \
        --prefix="$out" \
        --components=rust-std-${androidTarget}

      runHook postInstall
    '';
  };
  darwinHostLinkAttrs = lib.optionalAttrs stdenv.buildPlatform.isDarwin {
    RUSTFLAGS = "-L native=${buildPackages.libiconv}/lib";
    env.LIBRARY_PATH = "${buildPackages.libiconv}/lib";
  };
in
rec {
  default = rdict;

  rdict = rustPlatform.buildRustPackage ({
    inherit version src cargoHash;

    pname = "rdict";

    buildAndTestSubdir = "./rdict-cli";

    nativeBuildInputs =
      lib.optionals stdenv.buildPlatform.isDarwin [
        libiconv
      ]
      ++ lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
        installShellFiles
        writableTmpDirAsHomeHook
      ];

    postInstall = lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
      installShellCompletion --cmd rdict \
        --bash <("$out/bin/rdict" --completion bash) \
        --zsh <("$out/bin/rdict" --completion zsh) \
        --fish <("$out/bin/rdict" --completion fish)
    '';

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict";
    };
  } // darwinHostLinkAttrs);

  rdict-telegram = rustPlatform.buildRustPackage ({
    inherit version src cargoHash;

    pname = "rdict-telegram";

    nativeBuildInputs =
      lib.optionals stdenv.buildPlatform.isDarwin [
        libiconv
      ]
      ++ lib.optionals (stdenv.buildPlatform.canExecute stdenv.hostPlatform) [
        writableTmpDirAsHomeHook
      ];

    buildAndTestSubdir = "./rdict-telegram";

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict-telegram";
    };
  } // darwinHostLinkAttrs);

  rdict-iced = rustPlatform.buildRustPackage ({
    inherit version src cargoHash;

    pname = "rdict-iced";

    buildAndTestSubdir = "./rdict-iced";
    doCheck = stdenv.buildPlatform.canExecute stdenv.hostPlatform;

    nativeBuildInputs = lib.optionals stdenv.buildPlatform.isDarwin [
      libiconv
    ];

    meta = {
      license = lib.licenses.mit;
      mainProgram = "rdict-iced";
    };
  } // darwinHostLinkAttrs);

  rdict-iced-android = rustPlatform.buildRustPackage {
    inherit version src cargoHash;

    pname = "rdict-iced-android";

    nativeBuildInputs = [
      cargo-apk
      jdk_headless
    ];

    doCheck = false;

    prePatch = ''
      find_crate_vendor() {
        local crate="$1"
        local vendor

        vendor="$(find "$cargoDepsCopy" -type d -name "$crate-*" -print -quit)"
        if [ -z "$vendor" ]; then
          echo "Could not find vendored $crate crate" >&2
          exit 1
        fi

        printf '%s\n' "$vendor"
      }

      apply_crate_patch() {
        local crate="$1"
        local patch_file="$2"
        local vendor

        vendor="$(find_crate_vendor "$crate")"
        echo "Applying $(basename "$patch_file") to $crate"
        patch --batch -d "$vendor" -p1 < "$patch_file"
      }

      apply_crate_patch iced_material "${./nix/patches/iced-material-disable-iced-default-features.patch}"
      apply_crate_patch iced "${./nix/patches/iced-disable-wayland-on-android.patch}"
      apply_crate_patch iced_winit "${./nix/patches/iced-winit-android-app-lifecycle.patch}"
      apply_crate_patch winit "${./nix/patches/winit-android-ime.patch}"
    '';

    buildPhase = ''
      runHook preBuild

      export ANDROID_HOME="${androidSdk}/libexec/android-sdk"
      export ANDROID_SDK_ROOT="$ANDROID_HOME"
      export ANDROID_NDK_ROOT="$ANDROID_HOME/ndk-bundle"
      export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
      export JAVA_HOME="${jdk_headless}"
      export HOME="$TMPDIR"
      export PATH="${androidRustToolchain}/bin:$PATH"
      export RUSTC="${androidRustToolchain}/bin/rustc"

      key_store="$TMPDIR/rdict-debug.keystore"
      keytool -genkeypair \
        -keystore "$key_store" \
        -storepass android \
        -keypass android \
        -alias androiddebugkey \
        -keyalg RSA \
        -keysize 2048 \
        -validity 10000 \
        -dname "CN=Android Debug,O=Android,C=US" \
        -storetype PKCS12

      export CARGO_APK_RELEASE_KEYSTORE="$key_store"
      export CARGO_APK_RELEASE_KEYSTORE_PASSWORD=android

      cargo apk build \
        --release \
        --lib \
        -p rdict_iced \
        --target ${androidTarget} \
        --target-dir target/android \
        --manifest-path "$PWD/rdict-iced/Cargo.toml"

      apk="$(find target/android/release/apk -type f -name '*.apk' -print -quit)"
      if [ -z "$apk" ]; then
        echo "Could not find generated APK" >&2
        find target/android -maxdepth 5 -type f >&2 || true
        exit 1
      fi

      runHook postBuild
    '';

    installPhase = ''
      runHook preInstall

      apk="$(find target/android/release/apk -type f -name '*.apk' -print -quit)"
      if [ -z "$apk" ]; then
        echo "Could not find generated APK" >&2
        find target/android -maxdepth 5 -type f >&2 || true
        exit 1
      fi

      mkdir -p "$out"
      cp "$apk" "$out/rdict-iced.apk"

      runHook postInstall
    '';

    meta = {
      license = lib.licenses.mit;
    };
  };
}
