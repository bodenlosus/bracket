{
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      with nixpkgs.legacyPackages.${system};
      let
        pyproject = builtins.fromTOML (builtins.readFile ./pyproject.toml);

        pkg = python3.pkgs.buildPythonPackage rec {
          pname = pyproject.project.name;
          version = pyproject.project.version;
          format = "pyproject";
          src = ./.;

          nativeBuildInputs = with pkgs;
            [ gobject-introspection ] ++ (with python3.pkgs; [ setuptools tree-sitter]);

          propagatedBuildInputs = with pkgs;
            [
              gobject-introspection
              gtk4
              libadwaita
              libpanel
              pkg-config

            ] ++ (with python3.pkgs; [ pygobject3 pygobject-stubs pycairo ]);

          installCheckInputs = with python3.pkgs; [ ];
        };

        editablePkg = pkg.overrideAttrs (oldAttrs: {
          nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
            (python3.pkgs.mkPythonEditablePackage {
              pname = pyproject.project.name;
              inherit (pyproject.project) scripts version;
              root = "$PWD";
            })
          ];
        });

      in {
        packages.default = pkg;
        devShells.default = pkgs.mkShell {
          venvDir = "./.venv";
          packages = with pkgs;
            [ gobject-introspection pkg-config gtk4 libadwaita libpanel]
            ++ (with python3.pkgs; [ pygobject3 pygobject-stubs pycairo tree-sitter]);
          inputsFrom = [ editablePkg ];
        };
      });
}
