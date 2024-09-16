//! This module implements the necessary traits required to make `crate::Vite`
//! callable in tera templates.

use crate::vite::Vite;

use std::collections::HashMap;
use tera::{from_value, to_value, Function, Result, Value};

/// Allows for instances of Vite to be bound as a function.
///
/// # Examples
///
/// ```
/// use in_vite::Vite;
/// use tera::{Tera, Context, Result};
///
/// fn main() -> Result<()> {
///     let vite = Vite::default();
///     let mut tera = Tera::default();
///     tera.register_function("vite", vite);
///
///     let ctx = Context::new();
///     let template = tera.render_str(r#"{{ vite(resources="app.js" }}"#, &ctx);
///
///     Ok(())
/// }
///
/// ```
///
impl Function for Vite {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
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
                "The argument 'resources' must be either a string or an array of strings.".into(),
            );
        }

        let entrypoints = entrypoints.iter().map(|e| e.as_str()).collect();
        let code = self.to_html(entrypoints).unwrap();

        Ok(to_value(code)?)
    }
}

#[cfg(test)]
mod test {
    use crate::vite::{ViteOptions, ViteMode};

    use super::Vite;

    const SAMPLE_MANIFEST: &str = include_str!("../../test/sample_manifest.json");

    #[test]
    fn can_tera_inject_development() {
        let opts = ViteOptions::default()
            .mode(ViteMode::Development)
            .source(Some(SAMPLE_MANIFEST.to_string()));

        let vite = Vite::with_options(opts);
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
        let opts = ViteOptions::default()
            .mode(ViteMode::Production)
            .source(Some(SAMPLE_MANIFEST.to_string()));

        let vite = Vite::with_options(opts);
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
