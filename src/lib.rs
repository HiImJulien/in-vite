//! This module implements the necessary types and function required to
//! integrate Vite into Rust backend projects.

mod lib {
    use crate::vite5::Manifest;

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error(transparent)]
        Io(#[from] std::io::Error),
        #[error(transparent)]
        Json(#[from] serde_json::Error),
    }

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub(crate) enum Resource<'a> {
        Stylesheet(&'a str),
        Module(&'a str),
        PreloadModule(&'a str),
    }

    impl<'a> Resource<'a> {
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

    /// Determines whether the application is running in development mode.
    ///
    /// This function follows the conventions are provided by loco.rs, where
    /// it first checks `LOCO_ENV`, `RAILS_ENV`, `NODE_ENV`.
    ///
    /// When nothing is given, the default value is true.
    pub(crate) fn is_development_mode() -> bool {
        let mut mode = std::env::var("LOCO_ENV");
        if mode.is_err() {
            mode = std::env::var("RAILS_ENV");
        }

        if mode.is_err() {
            mode = std::env::var("NODE_ENV");
        }

        let mode = match mode {
            Ok(mode) => mode,
            Err(_) => "development".to_string(),
        };

        return mode == "development";
    }

    /// Encapsulates the default configuration options as well as
    /// the logic for resolving resources from vite.
    #[derive(Debug)]
    pub struct Vite {
        pub is_development_mode: Option<bool>,
        pub host: String,
        pub manifest: Option<String>,
    }

    impl<'a> Vite {
        pub fn new(
            is_development_mode: Option<bool>,
            host: String,
            manifest: Option<String>,
        ) -> Self {
            Vite {
                is_development_mode,
                host,
                manifest,
            }
        }

        fn make_development_html(&'a self) -> String {
            let host = &self.host;
            vec![
                // "<!-- Vite Development -->".to_string(),
                format!(r#"<script type="module" src="{host}/@vite/client"></script>"#),
                format!(r#"<script type="module" src="{host}/@vite/main.js"></script>"#),
            ]
            .join("\n")
        }

        pub fn make_html(&'a self, entrypoints: Vec<&'a str>) -> Result<String, Error> {
            let is_development_mode = self
                .is_development_mode
                .unwrap_or_else(|| is_development_mode());

            if is_development_mode {
                return Ok(self.make_development_html());
            }

            let manifest: Manifest = match &self.manifest {
                Some(manifest) => serde_json::from_str(&manifest)?,
                None => {
                    let file = std::fs::File::open("dist/.vite/manifest.json")?;
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
    }

    impl Default for Vite {
        fn default() -> Self {
            Vite {
                is_development_mode: None,
                host: "http://localhost:5173".to_string(),
                manifest: None,
            }
        }
    }
}

mod vite5 {
    use super::lib::Resource;
    use std::collections::HashMap;

    #[allow(dead_code)]
    #[derive(serde::Deserialize)]
    #[serde(transparent)]
    pub(crate) struct Manifest(HashMap<String, Chunk>);

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
            } else if key.ends_with(".js") {
                resources.push(Resource::Module(&chunk.file));
            }
        }
    }

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
}

#[cfg(test)]
mod test_vite5 {
    use crate::lib::Resource;

    use super::vite5::Manifest;

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

#[cfg(feature = "tera")]
pub mod tera {

    use std::collections::HashMap;
    use tera::{from_value, to_value, Function, Result, Value};

    use crate::lib::Vite;

    impl Function for Vite {
        fn call(&self, args: &HashMap<String, tera::Value>) -> Result<Value> {
            let Some(resources) = args.get("resources") else {
                return Err("Missing argument 'resources' in vite function.".into());
            };

            let entrypoints: Vec<String>;
            if resources.is_array() {
                entrypoints = from_value(resources.clone())?;
            } else if resources.is_string() {
                entrypoints = vec![from_value(resources.clone())?];
            } else {
                return Err(
                    "The argument 'resources' must be either a string or an array of strings."
                        .into(),
                );
            }

            let entrypoints = entrypoints.iter().map(|e| e.as_str()).collect();
            let code = self.make_html(entrypoints).unwrap();

            Ok(to_value(code)?)
        }

        fn is_safe(&self) -> bool {
            true
        }
    }
}

#[cfg(all(feature = "tera", test))]
mod test_tera {
    use super::lib::Vite;

    const SAMPLE_MANIFEST: &str = include_str!("../test/sample_manifest.json");

    #[test]
    fn can_tera_inject_development() {
        let vite = Vite::new(Some(true), "http://localhost:5173".to_string(), None);
        let mut tera = tera::Tera::default();

        tera.register_function("vite", vite);
        let result = tera.render_str(r#"{{ vite(resources="app.js") }}"#, &tera::Context::new());
        let expected = r#"<script type="module" src="http://localhost:5173/@vite/client"></script>
<script type="module" src="http://localhost:5173/@vite/main.js"></script>"#;

        assert!(matches!(result, Ok(_)));
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn can_tera_inject_production() {
        let vite = Vite::new(
            Some(false),
            "http://localhost:5173".to_string(),
            Some(SAMPLE_MANIFEST.to_string()),
        );
        let mut tera = tera::Tera::default();

        tera.register_function("vite", vite);
        let result = tera.render_str(
            r#"{{ vite(resources="views/foo.js") }}"#,
            &tera::Context::new(),
        );

        let expected = r#"<link rel="stylesheet" href="assets/foo-5UjPuW-k.css" />
<link rel="stylesheet" href="assets/shared-ChJ_j-JJ.css" />
<script type="module" src="assets/foo-BRBmoGS9.js"></script>
<link rel="modulepreload" href="assets/shared-B7PI925R.js" />"#;

        assert!(matches!(result, Ok(_)));
        assert_eq!(result.unwrap(), expected);
    }
}

#[cfg(feature = "minijinja")]
pub mod minijinja {

    use minijinja::value::{from_args, Kwargs, Object, ObjectRepr};
    use minijinja::{Error, Value};
    use std::sync::Arc;

    use crate::lib::Vite;

    type Result<T> = std::result::Result<T, Error>;

    impl Object for Vite {
        fn repr(self: &Arc<Self>) -> ObjectRepr {
            ObjectRepr::Plain
        }

        fn call(
            self: &Arc<Self>,
            _state: &minijinja::State<'_, '_>,
            args: &[minijinja::Value],
        ) -> Result<Value> {
            let (_, kwargs) = from_args::<(&[Value], Kwargs)>(&args)?;

            // The resources passed here are treated as entrypoint for vite.
            let entrypoints: Vec<String> = kwargs.get("resources")?;
            let entrypoints = entrypoints.iter().map(|e| e.as_str()).collect();

            let code = self.make_html(entrypoints).unwrap();
            Ok(Value::from(code))
        }

        fn is_true(self: &Arc<Self>) -> bool {
            true
        }
    }
}

#[cfg(all(feature = "minijinja", test))]
pub mod test_minijina {
    use super::lib::Vite;
    use minijinja::Environment;
    use minijinja::Value;

    const SAMPLE_MANIFEST: &str = include_str!("../test/sample_manifest.json");

    #[test]
    fn can_minijinja_inject_development() {
        let vite = Vite::new(Some(true), "http://localhost:5173".to_string(), Some(SAMPLE_MANIFEST.to_string()));
        let mut env = Environment::new();
        env.add_global("vite", Value::from_object(vite));
        let result = env
            .render_str(r#"{{ vite(resources=["views/foo.js"]) }}"#, Value::UNDEFINED)
            .expect("Should work.");

        let expected = r#"<script type="module" src="http://localhost:5173/@vite/client"></script>
<script type="module" src="http://localhost:5173/@vite/main.js"></script>"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn can_minijinja_inject_production() {
        let vite = Vite::new(Some(false), "http://localhost:5173".to_string(), Some(SAMPLE_MANIFEST.to_string()));
        let mut env = Environment::new();
        env.add_global("vite", Value::from_object(vite));
        let result = env
            .render_str(r#"{{ vite(resources=["views/foo.js"]) }}"#, Value::UNDEFINED)
            .expect("Should work.");

        let expected = r#"<link rel="stylesheet" href="assets/foo-5UjPuW-k.css" />
<link rel="stylesheet" href="assets/shared-ChJ_j-JJ.css" />
<script type="module" src="assets/foo-BRBmoGS9.js"></script>
<link rel="modulepreload" href="assets/shared-B7PI925R.js" />"#;

        assert_eq!(result, expected);
    }
}
