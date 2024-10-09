//! This module implements the necessary traits required to make `crate::Vite`
//! callable in tera templates.

use crate::vite::{Vite, ViteReactRefresh};

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

/// Allows for instances of ViteReactRefresh to be bound as a function.
///
/// # Examples
///
/// ```
/// use in_vite::{Vite, ViteReactRefresh};
/// use tera::{Tera, Context, Result};
///
/// fn main() -> Result<()> {
///     let vite = Vite::default();
///     let mut tera = Tera::default();
///     let vite_react_refresh = ViteReactRefresh::new(vite.host(), vite.mode());
///     tera.register_function("vite_react_refresh", vite_react_refresh);
///
///     let ctx = Context::new();
///     let template = tera.render_str(r#"{{ vite_react_refresh() }}"#, &ctx);
///
///     Ok(())
/// }
///
/// ```
///
impl Function for ViteReactRefresh {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, _args: &HashMap<String, Value>) -> Result<Value> {
        Ok(to_value(self.react_refresh())?)
    }
}

#[cfg(test)]
mod test {
    use crate::vite::{ViteMode, ViteOptions};

    use super::{Vite, ViteReactRefresh};

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
<script type="module" src="http://localhost:5173/app.js"></script>"#;

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

    #[test]
    fn can_tera_inject_react_refresh_development() {
        let opts = ViteOptions::default()
            .mode(ViteMode::Development)
            .source(Some(SAMPLE_MANIFEST.to_string()));

        let vite = Vite::with_options(opts);
        let vite_react_refresh = ViteReactRefresh::new(vite.host(), vite.mode());
        let mut tera = tera::Tera::default();

        tera.register_function("vite_react_refresh", vite_react_refresh);
        let result = tera.render_str(r#"{{ vite_react_refresh() }}"#, &tera::Context::new());
        let expected = r#"<script type="module">
import RefreshRuntime from "http://localhost:5173/@react-refresh"
RefreshRuntime.injectIntoGlobalHook(window)
window.$RefreshReg$ = () => {}
window.$RefreshSig$ = () => (type) => type
window.__vite_plugin_react_preamble_installed__ = true
</script>"#;

        assert!(matches!(result, Ok(_)));
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn tera_injects_nothing_react_refresh_production() {
        let opts = ViteOptions::default()
            .mode(ViteMode::Production)
            .source(Some(SAMPLE_MANIFEST.to_string()));

        let vite = Vite::with_options(opts);
        let vite_react_refresh = ViteReactRefresh::new(vite.host(), vite.mode());
        let mut tera = tera::Tera::default();

        tera.register_function("vite_react_refresh", vite_react_refresh);
        let result = tera.render_str(r#"{{ vite_react_refresh() }}"#, &tera::Context::new());
        let expected = "";

        assert!(matches!(result, Ok(_)));
        assert_eq!(result.unwrap(), expected);
    }
}
