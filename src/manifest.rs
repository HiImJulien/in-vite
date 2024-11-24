//! Implements the types `Manifest` and `Chunk`, required to deserialize
//! Vite's build manifest.
//!
//! For more information regarding Vite's build manifest see here
//!               https://vitejs.dev/guide/backend-integration
//! and here
//!             https://github.com/vitejs/vite/discussions/11546
//!

use std::collections::HashMap;

use crate::resource::Resource;

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(transparent)]
pub(crate) struct Manifest(HashMap<String, Chunk>);

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Chunk {
    #[serde(default)]
    pub src: Option<String>,

    pub file: String,

    #[serde(default)]
    pub css: Vec<String>,

    #[serde(default)]
    pub assets: Vec<String>,

    #[serde(default)]
    pub is_entry: bool,

    #[serde(default)]
    pub is_dynamic_entry: bool,

    #[serde(default)]
    pub imports: Vec<String>,

    #[serde(default)]
    pub dynamic_imports: Vec<String>,
}

impl<'a> Manifest {
    /// Returns a list of resources required to include given entrypoint.
    pub fn resolve_resources(&'a self, entrypoint: &'a str) -> Vec<Resource<'a>> {
        let Some(chunk) = self.0.get(entrypoint) else {
            return vec![];
        };

        if !chunk.is_entry {
            return vec![];
        }

        let mut resources: Vec<Resource<'a>> = vec![];
        self.resolve_imports(&mut resources, entrypoint, chunk);

        // Sorts the resources into following order:
        // 1. stylesheets
        // 2. modules
        // 3. preload modules
        resources.sort();
        resources
    }

    /// Recursively iterates through chunks and populates `resources`
    /// with the resources required.
    fn resolve_imports(
        &'a self,
        resources: &mut Vec<Resource<'a>>,
        key: &'a str,
        chunk: &'a Chunk,
    ) {
        for css in chunk.css.iter() {
            resources.push(Resource::Stylesheet(css));
        }

        for import in chunk.imports.iter() {
            let Some(chunk) = self.0.get(import) else {
                continue;
            };

            self.resolve_imports(resources, import, chunk);
        }

        // If the chunk is not a entrypoint, it may (optionally) be
        // preloaded.
        if !chunk.is_entry {
            resources.push(Resource::PreloadModule(&chunk.file));
            return;
        }

        if key.ends_with(".css") {
            resources.push(Resource::Stylesheet(&chunk.file));
        } else if key.ends_with(".js") || key.ends_with(".jsx") || key.ends_with(".ts") || key.ends_with(".tsx") {
            resources.push(Resource::Module(&chunk.file));
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Manifest, Resource};

    const SAMPLE_MANIFEST: &str = include_str!("../test/sample_manifest.json");

    #[test]
    fn can_deserialize_sample_manifest() {
        let result = serde_json::from_str::<Manifest>(SAMPLE_MANIFEST);
        assert!(result.is_ok());
    }

    #[test]
    fn can_resolve_entrypoints() {
        let manifest = serde_json::from_str::<Manifest>(SAMPLE_MANIFEST)
            .expect("sample manifest should be deserializable");

        let resources = manifest.resolve_resources("views/foo.js");
        let expected = vec![
            Resource::Stylesheet("assets/foo-5UjPuW-k.css"),
            Resource::Stylesheet("assets/shared-ChJ_j-JJ.css"),
            Resource::Module("assets/foo-BRBmoGS9.js"),
            Resource::PreloadModule("assets/shared-B7PI925R.js"),
        ];

        assert_eq!(resources, expected);
    }
}
