use anyhow::anyhow;
use std::path::PathBuf;

/// Expand a leading `~` or `~/` in a path to the value of `$HOME`.
pub fn expand_tilde(path: &str) -> anyhow::Result<PathBuf> {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow!("Could not determine $HOME for tilde expansion"))?;
        Ok(PathBuf::from(home).join(rest))
    } else if path == "~" {
        let home = std::env::var("HOME")
            .map_err(|_| anyhow!("Could not determine $HOME for tilde expansion"))?;
        Ok(PathBuf::from(home))
    } else {
        Ok(PathBuf::from(path))
    }
}

/// Returns the path to the cache file at `$XDG_CACHE_HOME/homage/cache.toml`.
/// Falls back to `$HOME/.cache/homage/cache.toml` if `XDG_CACHE_HOME` is not set.
pub fn cache_path() -> anyhow::Result<PathBuf> {
    let cache_home = match std::env::var("XDG_CACHE_HOME") {
        Ok(val) if !val.is_empty() => PathBuf::from(val),
        _ => {
            let home = std::env::var("HOME")
                .map_err(|_| anyhow!("Could not determine $HOME for cache path"))?;
            PathBuf::from(home).join(".cache")
        }
    };

    Ok(cache_home.join("homage").join("cache.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tilde_expansion() {
        let result = expand_tilde("~/config/file").unwrap();
        let home = std::env::var("HOME").unwrap();
        assert_eq!(result, PathBuf::from(home).join("config/file"));
    }

    #[test]
    fn tilde_alone_expands() {
        let result = expand_tilde("~").unwrap();
        let home = std::env::var("HOME").unwrap();
        assert_eq!(result, PathBuf::from(home));
    }

    #[test]
    fn absolute_path_unchanged() {
        let result = expand_tilde("/absolute/path").unwrap();
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }
}
