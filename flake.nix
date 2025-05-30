{
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";
  inputs.highlighter.url = "path:./highlighter";

  outputs = inputs@{ self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = import nixpkgs { system = "x86_64-linux"; };
        pyproject = builtins.fromTOML (builtins.readFile ./pyproject.toml);
        python-bin = pkgs.python312;
        highlighter = python-bin.pkgs.callPackage ./highlighter/package.nix {};
        nativePkgs = [highlighter] ++ (with pkgs; [ gobject-introspection ]);
        nativePythonPkgs = with python-bin.pkgs; [ setuptools ];

        propagatedPkgs = with pkgs; [ gtk4 libadwaita libpanel pkg-config ];

        devPkgs = (with pkgs; [
          flatpak-builder
          libxml2
          blueprint-compiler
          pkg-config
          meson
          ninja
          basedpyright
          openssl
          mypy
        ]) ++ (with python-bin.pkgs; [ virtualenv ]);
        propagatedPythonPkgs = with python-bin.pkgs; [
          pygobject3
          pygobject-stubs
          pycairo
        ];

        pkg = python-bin.pkgs.buildPythonPackage {
          pname = pyproject.project.name;
          version = pyproject.project.version;
          format = "pyproject";
          src = ./.;

          nativeBuildInputs = nativePkgs ++ nativePythonPkgs
            ++ [ pkgs.wrapGAppsHook ];

          propagatedBuildInputs = propagatedPkgs ++ propagatedPythonPkgs;

          installCheckInputs = with python-bin.pkgs; [ ];
        };

        editablePkg = pkg.overrideAttrs (oldAttrs: {
          nativeBuildInputs = oldAttrs.nativeBuildInputs ++ [
            (python-bin.pkgs.mkPythonEditablePackage {
              pname = pyproject.project.name;
              # inherit (pyproject.project) scripts version;
              inherit (pyproject.project) version;
              root = "$PWD";
            })
          ];
        });

      in {
        packages.default = pkg;
        devShells.default = pkgs.mkShell {
          venvDir = "./.venv";
          packages = nativePkgs ++ nativePythonPkgs ++ propagatedPkgs
            ++ propagatedPythonPkgs ++ devPkgs;
          shellHook = ''
            export XDG_DATA_DIRS=${pkgs.gtk4}/share/gsettings-schemas/gtk4-${pkgs.gtk4.version}:$XDG_DATA_DIRS
          '';
          inputsFrom = [ editablePkg ];
        };
      });
}
