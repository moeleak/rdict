{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "rdict";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  useFetchCargoVendor = true;
  cargoHash = "sha256-nGHGfpAm73KkEOv5IxZapyAWbhGb4WuH98wxsOWImZY=";

  preCheck = ''
    export HOME="$(mktemp -d)"
  '';

  meta = {
    license = lib.licenses.mit;
  };
}
