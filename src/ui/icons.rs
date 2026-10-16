use ratatui::style::Color;
use std::path::Path;

pub struct FileIcon {
    pub glyph: &'static str,
    pub color: Color,
}

/// Resolves authentic Nerd Font icons and Catppuccin colors based on filename/extension
pub fn get_file_icon(path: &Path, is_dir: bool, is_encrypted: bool) -> FileIcon {
    if is_encrypted {
        return FileIcon {
            glyph: "󰌆 ",
            color: Color::Rgb(250, 179, 135), // Peach
        };
    }

    if is_dir {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        return match name.as_str() {
            ".git" => FileIcon {
                glyph: " ",
                color: Color::Rgb(243, 139, 168),
            }, // Maroon
            "node_modules" => FileIcon {
                glyph: " ",
                color: Color::Rgb(166, 227, 161),
            }, // Green
            "src" => FileIcon {
                glyph: "󰚝 ",
                color: Color::Rgb(137, 180, 250),
            }, // Blue
            "target" | "build" | "dist" => FileIcon {
                glyph: "󰏗 ",
                color: Color::Rgb(249, 226, 175),
            }, // Yellow
            "assets" | "static" | "public" => FileIcon {
                glyph: "󰋩 ",
                color: Color::Rgb(203, 166, 247),
            }, // Mauve
            ".cargo" | ".rustup" => FileIcon {
                glyph: " ",
                color: Color::Rgb(250, 179, 135),
            }, // Peach
            "tests" => FileIcon {
                glyph: "󰙨 ",
                color: Color::Rgb(148, 226, 213),
            }, // Teal
            _ => FileIcon {
                glyph: "󰉋 ",
                color: Color::Rgb(137, 180, 250),
            }, // Default Dir Blue
        };
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    if name == "cargo.toml" || name == "cargo.lock" {
        return FileIcon {
            glyph: " ",
            color: Color::Rgb(250, 179, 135),
        };
    }
    if name == "dockerfile" || name == "docker-compose.yml" {
        return FileIcon {
            glyph: "󰡨 ",
            color: Color::Rgb(116, 199, 236),
        };
    }
    if name.starts_with(".git") {
        return FileIcon {
            glyph: " ",
            color: Color::Rgb(243, 139, 168),
        };
    }

    match ext.as_str() {
        // Programming languages
        "rs" => FileIcon {
            glyph: " ",
            color: Color::Rgb(250, 179, 135),
        },
        "py" | "pyc" | "ipynb" => FileIcon {
            glyph: " ",
            color: Color::Rgb(249, 226, 175),
        },
        "js" | "mjs" | "cjs" => FileIcon {
            glyph: " ",
            color: Color::Rgb(249, 226, 175),
        },
        "ts" | "tsx" => FileIcon {
            glyph: " ",
            color: Color::Rgb(116, 199, 236),
        },
        "jsx" => FileIcon {
            glyph: " ",
            color: Color::Rgb(116, 199, 236),
        },
        "c" | "h" => FileIcon {
            glyph: " ",
            color: Color::Rgb(137, 180, 250),
        },
        "cpp" | "hpp" | "cc" => FileIcon {
            glyph: " ",
            color: Color::Rgb(116, 199, 236),
        },
        "go" => FileIcon {
            glyph: " ",
            color: Color::Rgb(148, 226, 213),
        },
        "java" | "class" | "jar" => FileIcon {
            glyph: " ",
            color: Color::Rgb(243, 139, 168),
        },
        "html" | "htm" => FileIcon {
            glyph: " ",
            color: Color::Rgb(250, 179, 135),
        },
        "css" | "scss" | "sass" => FileIcon {
            glyph: " ",
            color: Color::Rgb(137, 180, 250),
        },
        "sh" | "bash" | "zsh" | "fish" => FileIcon {
            glyph: " ",
            color: Color::Rgb(166, 227, 161),
        },
        "lua" => FileIcon {
            glyph: " ",
            color: Color::Rgb(137, 180, 250),
        },
        "rb" => FileIcon {
            glyph: " ",
            color: Color::Rgb(243, 139, 168),
        },
        "php" => FileIcon {
            glyph: " ",
            color: Color::Rgb(203, 166, 247),
        },
        "sql" => FileIcon {
            glyph: " ",
            color: Color::Rgb(249, 226, 175),
        },

        // Data & Config
        "json" => FileIcon {
            glyph: " ",
            color: Color::Rgb(249, 226, 175),
        },
        "yaml" | "yml" => FileIcon {
            glyph: " ",
            color: Color::Rgb(243, 139, 168),
        },
        "toml" => FileIcon {
            glyph: " ",
            color: Color::Rgb(250, 179, 135),
        },
        "xml" | "svg" => FileIcon {
            glyph: "󰗀 ",
            color: Color::Rgb(249, 226, 175),
        },
        "csv" | "tsv" => FileIcon {
            glyph: "󰈙 ",
            color: Color::Rgb(166, 227, 161),
        },
        "ini" | "conf" | "cfg" => FileIcon {
            glyph: " ",
            color: Color::Rgb(166, 173, 200),
        },

        // Documentation & Text
        "md" | "markdown" => FileIcon {
            glyph: " ",
            color: Color::Rgb(148, 226, 213),
        },
        "txt" => FileIcon {
            glyph: "󰈙 ",
            color: Color::Rgb(205, 214, 244),
        },
        "pdf" => FileIcon {
            glyph: " ",
            color: Color::Rgb(243, 139, 168),
        },

        // Media & Images
        "png" | "jpg" | "jpeg" | "gif" | "webp" => FileIcon {
            glyph: "󰋩 ",
            color: Color::Rgb(203, 166, 247),
        },
        "mp4" | "mkv" | "avi" | "mov" => FileIcon {
            glyph: "󰿎 ",
            color: Color::Rgb(243, 139, 168),
        },
        "mp3" | "wav" | "flac" | "ogg" => FileIcon {
            glyph: "󰎆 ",
            color: Color::Rgb(148, 226, 213),
        },

        // Archives
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "zst" => FileIcon {
            glyph: " ",
            color: Color::Rgb(250, 179, 135),
        },

        // Binaries / Executables
        "so" | "a" | "dll" | "dylib" => FileIcon {
            glyph: "󰌆 ",
            color: Color::Rgb(166, 173, 200),
        },
        "exe" | "bin" => FileIcon {
            glyph: "󰜎 ",
            color: Color::Rgb(166, 227, 161),
        },

        // Fallback
        _ => FileIcon {
            glyph: "󰈔 ",
            color: Color::Rgb(205, 214, 244),
        },
    }
}
