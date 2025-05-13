{ lib, rustPlatform }:
rustPlatform.buildRustPackage {
  pname = "rdict";
  version = "0.1.0";

  src = lib.cleanSource ./.;

  useFetchCargoVendor = true;
  cargoHash = "sha256-8aqVyPEpeFr8zo6r7bLMUR2nEnwtw87v+jt4iSdjn18=";

  preCheck = ''
    export HOME="$(mktemp -d)"
  '';

  meta = {
    license = lib.licenses.mit;
  };
}
