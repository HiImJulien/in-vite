# in-vite

Integrate vite into your backend.

## Installing

This library can be added to your project simply by calling:

```sh
cargo add in-vite
```

## Integrations

This library includes integrations for template engines such as [tera](https://github.com/Keats/tera)
and [minijinja](https://github.com/mitsuhiko/minijinja). The integrations can
be activated by including the appropriate feature flag.

### Tera

To install with the tera integration simply call:

```sh
cargo add in-vite -F tera
```

And integrated simply so:

```rs

let vite = Vite::default();

let mut tera = tera::Tera::default();
tera.register_function("vite", vite);


let template = tera.render_str(
  r#"{{ vite(resources="views/foo.js") }}"#
)?;
```

### Minijinja

To install with the minijinja integration simply call:

```sh
cargo add in-vite -F minijinja
```

And integrated simply so:

```rs

let vite = Vite::default();

let mut env = minijinja::Environment::new();
env.add_global("vite", Value::from_object(vite));

let template = env.render_str(
  r#"{{ vite(resources=["views/foo.js"]) }}"#
)?;

```

