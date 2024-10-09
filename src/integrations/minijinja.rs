//! This module implements the necessary traits required to make `crate::Vite`
//! callable in minijinja templates.

use crate::vite::{Vite, ViteReactRefresh};

use std::sync::Arc;

use minijinja::value::{from_args, Kwargs, Object, ObjectRepr};
use minijinja::{Error, Value};

/// Allows for instances fof Vite to be bound as values and added to the
/// minijinja environment.
///
/// # Examples
///
/// ```
/// use in_vite::Vite;
/// use minijinja::{Environment, Value, Error};
///
/// fn main() -> Result<(), Error> {
///     let vite = Vite::default();
///     let mut env = Environment::new();
///     env.add_global("vite", Value::from_object(vite));
///
///     let template = env.render_str(r#""#, Value::UNDEFINED)?;
///     Ok(())
/// }
///
/// ```
///
impl Object for Vite {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(
        self: &Arc<Self>,
        _state: &minijinja::State<'_, '_>,
        args: &[minijinja::Value],
    ) -> Result<Value, Error> {
        let (_, kwargs) = from_args::<(&[Value], Kwargs)>(&args)?;

        // The resources passed here are treated as entrypoint for vite.
        let entrypoints: Vec<String> = kwargs.get("resources")?;
        let entrypoints = entrypoints.iter().map(|e| e.as_str()).collect();

        let code = self.to_html(entrypoints).unwrap();
        Ok(Value::from_safe_string(code))
    }

    fn is_true(self: &Arc<Self>) -> bool {
        true
    }
}

/// Allows for instances fo ViteReactRefresh to be bound as values and added to the
/// minijinja environment.
///
/// # Examples
///
/// ```
/// use in_vite::{Vite, ViteReactRefresh};
/// use minijinja::{Environment, Value, Error};
///
/// fn main() -> Result<(), Error> {
///     let vite = Vite::default();
///     let vite_react_refresh = ViteReactRefresh::new(vite.host(), vite.mode());
///     let mut env = Environment::new();
///     env.add_global("vite_react_refresh", Value::from_object(vite_react_refresh));
///
///     let template = env.render_str(r#"{{ vite_react_refresh() }}"#, Value::UNDEFINED)?;
///     Ok(())
/// }
///
/// ```
///
impl Object for ViteReactRefresh {
    fn repr(self: &Arc<Self>) -> ObjectRepr {
        ObjectRepr::Plain
    }

    fn call(
        self: &Arc<Self>,
        _state: &minijinja::State<'_, '_>,
        _args: &[minijinja::Value],
    ) -> Result<Value, Error> {
        let code = self.react_refresh();

        Ok(Value::from_safe_string(code))
    }
}

#[cfg(test)]
mod test {

    use super::{Vite, ViteReactRefresh};
    use crate::vite::{ViteMode, ViteOptions};
    use minijinja::Environment;
    use minijinja::Value;

    const SAMPLE_MANIFEST: &str = include_str!("../../test/sample_manifest.json");

    #[test]
    fn can_minijinja_inject_development() {
        let opts = ViteOptions::default()
            .mode(ViteMode::Development)
            .source(Some(SAMPLE_MANIFEST.to_string()));

        let vite = Vite::with_options(opts);
        let mut env = Environment::new();
        env.add_global("vite", Value::from_object(vite));
        let result = env
            .render_str(
                r#"{{ vite(resources=["views/foo.js"]) }}"#,
                Value::UNDEFINED,
            )
            .expect("Should work.");

        let expected = r#"<script type="module" src="http://localhost:5173/@vite/client"></script>
<script type="module" src="http://localhost:5173/views/foo.js"></script>"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn can_minijinja_inject_production() {
        let opts = ViteOptions::default()
            .mode(ViteMode::Production)
            .source(Some(SAMPLE_MANIFEST.to_string()));

        let vite = Vite::with_options(opts);
        let mut env = Environment::new();
        env.add_global("vite", Value::from_object(vite));
        let result = env
            .render_str(
                r#"{{ vite(resources=["views/foo.js"]) }}"#,
                Value::UNDEFINED,
            )
            .expect("Should work.");

        let expected = r#"<link rel="stylesheet" href="assets/foo-5UjPuW-k.css" />
<link rel="stylesheet" href="assets/shared-ChJ_j-JJ.css" />
<script type="module" src="assets/foo-BRBmoGS9.js"></script>
<link rel="modulepreload" href="assets/shared-B7PI925R.js" />"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn can_minijinja_inject_react_refresh_development() {
        let opts = ViteOptions::default()
            .mode(ViteMode::Development)
            .source(Some(SAMPLE_MANIFEST.to_string()));

        let vite = Vite::with_options(opts);
        let vite_react_refresh = ViteReactRefresh::new(vite.host(), vite.mode());
        let mut env = Environment::new();
        env.add_global("vite_react_refresh", Value::from_object(vite_react_refresh));
        let result = env
            .render_str(r#"{{ vite_react_refresh() }}"#, Value::UNDEFINED)
            .expect("Should work.");

        let expected = r#"<script type="module">
import RefreshRuntime from "http://localhost:5173/@react-refresh"
RefreshRuntime.injectIntoGlobalHook(window)
window.$RefreshReg$ = () => {}
window.$RefreshSig$ = () => (type) => type
window.__vite_plugin_react_preamble_installed__ = true
</script>"#;

        assert_eq!(result, expected);
    }

    #[test]
    fn minijinja_injects_nothing_react_refresh_production() {
        let opts = ViteOptions::default()
            .mode(ViteMode::Production)
            .source(Some(SAMPLE_MANIFEST.to_string()));

        let vite = Vite::with_options(opts);
        let vite_react_refresh = ViteReactRefresh::new(vite.host(), vite.mode());
        let mut env = Environment::new();
        env.add_global("vite_react_refresh", Value::from_object(vite_react_refresh));
        let result = env
            .render_str(r#"{{ vite_react_refresh() }}"#, Value::UNDEFINED)
            .expect("Should work.");

        let expected = "";

        assert_eq!(result, expected);
    }
}
