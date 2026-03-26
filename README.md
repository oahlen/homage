# Homage

Simple and effective dotfiles manager for your home.

Manages dotfiles through a simple manifest file format that declares which files should be symlinked and where.
Homage tracks installed symlinks in a cache so it can automatically clean up stale entries when a manifest changes.

## Manifest format

A manifest file specifies dotfiles to install and can include other manifests:

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
- **Directories** are traversed recursively and all contained files are individually symlinked,
  with intermediate directories created as needed.
- **Includes** reference other manifest files (paths relative to the including manifest) and are resolved recursively.
  Circular includes are detected and rejected.

## Usage

### Install

```sh
homage install manifest.toml
```

Parses the manifest, compares against the cache, removes symlinks that are no longer referenced, and installs new ones.
Running install again after modifying the manifest will automatically clean up entries that were removed.

### Uninstall

```sh
homage uninstall manifest.toml
```

Removes all managed symlinks referenced by the manifest and any remaining entries in the cache,
then deletes the cache file.

### Options

| Flag | Description |
|---|---|
| `--dry-run` | Preview changes without modifying the file system |
| `--no-confirm` | Skip the confirmation prompt |
| `-v` / `-vv` / `-vvv` | Increase output verbosity |
| `--quiet` | Only print error messages |

## Cache

Homage stores its state at `$XDG_CACHE_HOME/homage/cache.toml` (falls back to `$HOME/.cache/homage/cache.toml`).
The cache maps installed source files to their target locations so that stale entries can be detected and cleaned up on
subsequent installs.
