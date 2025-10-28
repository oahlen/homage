use std::path::Path;

use colored::{ColoredString, Colorize};

pub fn fmt_number(number: usize) -> ColoredString {
    number.to_string().magenta()
}

pub fn fmt_dir(path: &Path) -> ColoredString {
    path.display().to_string().blue().bold()
}

pub fn fmt_file(path: &Path) -> ColoredString {
    path.display().to_string().blue()
}

pub fn fmt_link(path: &Path) -> ColoredString {
    path.display().to_string().cyan()
}
