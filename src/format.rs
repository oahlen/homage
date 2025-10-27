use std::path::Path;

use colored::{ColoredString, Colorize};

pub fn format_dir(path: &Path) -> ColoredString {
    path.display().to_string().blue()
}

pub fn format_file(path: &Path) -> ColoredString {
    path.display().to_string().blue()
}

pub fn format_link(path: &Path) -> ColoredString {
    path.display().to_string().cyan()
}
