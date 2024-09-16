//! This module implements the type `Resource`.

/// Enumerates all resources bundled by Vite.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Resource<'a> {
    // Represents a CSS stylesheet to be loaded.
    Stylesheet(&'a str),

    // Represents a JavaScript module to be loaded.
    Module(&'a str),

    // Represents a JavaScript module, which can be preloaded
    // using Vite's preload polyfill.
    PreloadModule(&'a str),
}

impl<'a> Resource<'a> {

    /// Converts the resource into the appropriate HTML code required to include
    /// the resource.
    pub fn to_html(&'a self) -> String {
        match *self {
            Self::Stylesheet(uri) => format!(r#"<link rel="stylesheet" href="{uri}" />"#),
            Self::Module(uri) => format!(r#"<script type="module" src="{uri}"></script>"#),
            Self::PreloadModule(uri) => {
                format!(r#"<link rel="modulepreload" href="{uri}" />"#)
            }
        }
    }

}

