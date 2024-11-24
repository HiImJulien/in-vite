//! This module implements the type `Vite` and `ViteOptions`.

use crate::error::Error;
use crate::manifest::Manifest;
use crate::resource::Resource;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum ViteMode {
    #[default]
    Development,
    Production,
}

pub struct ViteOptions {
    pub(crate) host: String,
    pub(crate) manifest_source: Option<String>,
    pub(crate) manifest_path: String,
    pub(crate) mode: ViteMode,
}

impl Default for ViteOptions {
    fn default() -> Self {
        ViteOptions {
            host: "http://localhost:5173".to_string(),
            manifest_source: None,
            manifest_path: "dist/.vite/manifest.json".to_string(),
            mode: ViteMode::default(),
        }
        .guess_mode()
    }
}

impl ViteOptions {
    fn new() -> Self {
        ViteOptions {
            host: "".to_string(),
            manifest_source: None,
            manifest_path: "dist/.vite/manifest.json".to_string(),
            mode: ViteMode::default(),
        }
    }

    /// Sets the host, from which Vite's development scripts should be loaded.
    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    /// Sets the manifest source to deserialize the manifest from.
    pub fn source<S: Into<String>>(mut self, source: Option<S>) -> Self {
        self.manifest_source = source.and_then(|src| Some(src.into()));
        self
    }

    /// Sets the path from where to load and deserialize the manifest from.
    pub fn manifest_path<S: Into<String>>(mut self, path: S) -> Self {
        self.manifest_path = path.into();
        self
    }

    /// Sets the mode in which resources should be included.
    pub fn mode(mut self, mode: ViteMode) -> Self {
        self.mode = mode;
        self
    }

    /// Attempts to guess the mode from environment variables.
    ///
    /// This method looks for the following environment variables:
    /// - `LOCO_ENV`
    /// - `RAILS_ENV`
    /// - `NODE_ENV`
    ///
    /// and checks whether they evaluate to `development` or `production`.
    /// If neither can be found, assumes `development`.
    ///
    pub fn guess_mode(mut self) -> Self {
        let mode = std::env::var("LOCO_ENV")
            .or_else(|_| std::env::var("RAILS_ENV"))
            .or_else(|_| std::env::var("NODE_ENV"));

        let mode = mode.unwrap_or("development".to_string());
        self.mode = match mode.as_str() {
            "production" => ViteMode::Production,
            _ => ViteMode::Development,
        };

        self
    }
}

/// Encapsulates the configuration and logic required for resolving resources
/// bundled by vite.
#[derive(Debug)]
pub struct Vite {
    host: String,
    manifest_source: Option<String>,
    manifest_path: String,
    mode: ViteMode,
}

impl Default for Vite {
    fn default() -> Self {
        Self::with_options(ViteOptions::default())
    }
}

impl<'a> Vite {
    pub fn with_options(opts: ViteOptions) -> Self {
        Self {
            host: opts.host,
            manifest_source: opts.manifest_source,
            manifest_path: opts.manifest_path,
            mode: opts.mode,
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn mode(&self) -> &ViteMode {
        &self.mode
    }

    pub fn to_html(&'a self, entrypoints: Vec<&'a str>) -> Result<String, Error> {
        if self.mode == ViteMode::Development {
            return Ok(self.to_development_html(entrypoints));
        }

        let manifest: Manifest = match &self.manifest_source {
            Some(manifest) => serde_json::from_str(&manifest)?,
            None => {
                let file = std::fs::File::open(&self.manifest_path)?;
                serde_json::from_reader(file)?
            }
        };

        let mut resources: Vec<Resource> = entrypoints
            .iter()
            .map(|entrypoint| manifest.resolve_resources(entrypoint))
            .flatten()
            .collect();

        resources.sort();
        let html = resources
            .into_iter()
            .map(|resource| resource.to_html())
            .collect::<Vec<String>>()
            .join("\n");

        Ok(html)
    }

    fn to_development_html(&'a self, entrypoints: Vec<&'a str>) -> String {
        let host = &self.host;
        let mut lines: Vec<String> = vec![format!(
            r#"<script type="module" src="{host}/@vite/client"></script>"#
        )];

        entrypoints
            .iter()
            .map(|entry| format!(r#"<script type="module" src="{host}/{entry}"></script>"#))
            .for_each(|line| lines.push(line));

        lines.join("\n")
    }
}

#[derive(Debug)]
pub struct ViteReactRefresh {
    host: String,
    mode: ViteMode,
}

impl ViteReactRefresh {
    pub fn new<S: AsRef<str>>(host: S, mode: &ViteMode) -> Self {
        Self {
            host: host.as_ref().to_owned(),
            mode: mode.to_owned(),
        }
    }

    pub fn react_refresh(&self) -> String {
        if self.mode == ViteMode::Development {
            let host = &self.host;

            format!(
                r#"<script type="module">
import RefreshRuntime from "{host}/@react-refresh"
RefreshRuntime.injectIntoGlobalHook(window)
window.$RefreshReg$ = () => {{}}
window.$RefreshSig$ = () => (type) => type
window.__vite_plugin_react_preamble_installed__ = true
</script>"#
            )
        } else {
            "".to_string()
        }
    }
}
