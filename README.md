# Bracket - A minimalist code editor with the GNOME-Desktop in mind

Bracket aims to be a modern code editor for the linux desktop.
It was designed with the GNOME-Platform in mind and is therfore built on GTK4 and libadwaita.

It currently features
- fast syntax highlighting with treesitter, for python only
- a file-tree

## Installation
The following is required to run the project:
- a linux desktop environment supporting GTK4 (GNOME has been tested)
- a machine on x86_64

### Flatpak (recommended)
Install it via the `bracket.flatpak` file found in the releases.

### Manual (not tested)
#### Requirements
requires:
- meson
- python >= 3.12
- pygobject3
- pycairo

Build the app using meson:

```sh

git clone --depth 1 https://github.com/bodenlosus/bracket.git
cd bracket
meson setup builddir --prefix=$HOME/.local
meson install -C builddir

```

Then run the app with

```sh

bracket <DIRECTORY>

```

or

```sh

~/.local/bin/bracket <DIRECTORY>

```
