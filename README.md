# Homage

Simple and effective *dotfiles* manager for your home.

Homage manages your dotfiles through a simple manifest file format that declares which files should be symlinked and
where. Homage tracks previously installed symlinks and automatically cleans up stale entries when a manifest changes.

## Manifest format

A manifest file specifies dotfiles to install and can include other manifests. This is useful for declaring a set of
common dotfiles to be shared across multiple machines.

### Example

```toml
includes = [
  "common.toml",
  "../graphical-tools/manifest.toml",
]

[files]
"niri/config.kdl" = "~/.config/niri/config.kdl"
"waybar" = "~/.config/waybar"
```

- **Source paths** (left side) are resolved relative to the manifest file.
- **Target paths** (right side) support `~` for the home directory.
- **Individual files** are symlinked directly.
- **Directories** are traversed recursively and all contained files are individually symlinked, with intermediate
  directories created as needed.
- **Includes** reference other manifest files (paths relative to the including manifest) and are resolved recursively.
  Circular includes are detected and rejected.

## Usage

Below are the main commands of the program, see the `--help` flag for details.

### Install

```sh
homage install manifest.toml
```

Parses a dotfiles manifest and install all relevant files/symlinks.
Running install again after modifying the manifest will automatically clean up entries that were removed.
This operation is idempotent, running the command on the same manifest multiple times yields the exact same result.

### Uninstall

```sh
homage uninstall manifest.toml
```

Removes all managed symlinks referenced by the dotfiles manifest and any remaining stale entries.

## Cache

Homage stores its state at `$XDG_CACHE_HOME/homage/cache.toml` (falls back to `$HOME/.cache/homage/cache.toml`).
The cache maps installed source files to their target locations so that stale entries can be detected and cleaned up on
subsequent installs.
