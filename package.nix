{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "rdict";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  useFetchCargoVendor = true;
  cargoHash = "sha256-8+2IjvYBa2C49LcAycx38tw5DafN43Gnmt5l6+lN5Co=";

  preCheck = ''
    export HOME="$(mktemp -d)"
  '';

  meta = {
    license = lib.licenses.mit;
  };
}
