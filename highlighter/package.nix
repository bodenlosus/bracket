{
  lib,
  buildPythonPackage,
  maturin,
  rustPlatform,
  rustc,
  cargo
}:

buildPythonPackage {
  pname = "highlighter";
  version = "0.1.0";
  pyproject = true;

  src = ./.;

  cargoDeps = rustPlatform.fetchCargoVendor {
    src = ./.;
    hash = "sha256-pM1aUxrWmbKINAVxnFSRHmszMH/MZG8CLbkz6w/u5yg=";
  };

  nativeBuildInputs = with rustPlatform; [
    rustPlatform.cargoSetupHook
    maturinBuildHook
    maturin
    rustc
    cargo
  ];
}
