<div align="center">
  <h1>Welcome to in-vite :crab:</h1>
</div>

## What's in-vite?

`In-vite` is a small library, inspired by Laravel's Vite Plugin. It allows you
to integrate vite's bundling capabilities into your Rust :crab: backend.

## Getting Started :rocket:

```sh
cargo install in-vite
```

The library revolves around the struct `Vite` which handles most aspects and
is required for integration:

```rs

use in_vite::Vite;

fn main() {
  let vite = Vite::default();

  // Retrieve the HTML required to include app.js and it's dependencies.
  let code = vite.make_html(vec!["app.js"]).unwrap();
}

```

> [!IMPORTANT]
> `in-vite` does not setup Vite by itself, rather it expects an already
> setup isntance.
> On how to setup Vite read further.

### Setting up Vite :construction:

This library requires an instance of Vite to be already setup. To setup Vite
use your favorite package manager, for example using `npm`:

```sh
npm create vite@latest
```

Next, you need to extend Vite's `vite.config.js`:

```js
// vite.config.js

export default defineConfig({
  build: {
    manifest: true,
    rollupOptions: {
      input: 'app.js'
    },
  }
});

```

The manifest is used in production builds to resolve the appropriate
build artifact.

> [!NOTE]
> You must manually specify entrypoints, since Vite has no `index.html`
> to go from.

### Further configurations

TODO: Once the struct `ViteOptions` is implemented, document it here.

## Integrations :world_map:

`in-vite` provides integrations for templating engines such as
[tera](https://github.com/Keats/tera) and
[minijinja](https://github.com/mitsuhiko/minijinja). Which can be activated
using the appropriate feature flag.

### Integration with `tera`

Using the feature flag `tera`, the integration can be activated:

```sh
cargo add in-vite -F tera
```

Integrating Vite is as simple as registering a function with your `tera::Tera`
instance:

```rs

let vite = Vite::default();

let mut tera = tera::Tera::default();
tera.register_function("vite", vite);

let template = tera.render_str(r#"{{ vite(resources="app.js") }}"#, &tera::Context::new())?;

```

### Integration with `minijinja` :ninja:

Like other integrations, this one can be activated with the feature flag `minijinja`:

```sh
cargo add in-vite -F minijinja
```

```rs
let vite = Vite::default();

let mut env = minijinja::Environment::new();
env.add_global("vite", minijinja::Value::from_object(vite));

let template = env.render_str(r#"{{ vite(resources="app.js") }}"#, minijinja::Value::UNDEFINED)?;
```

## Contributing

If you consider contributing, then first of all: Thank you! :gift_heart:
The first and simplest way to show your support is to star this repo.

This project accepts bug reports and feature requests via the integrated
[issue tracker](https://github.com/HiImJulien/in-vite/issues). Pull requests
for new integrations are also welcome!

Additionally, code reviews and pointers on how to improve the libraries code
are welcome. This is my first Rust library after all.

## Sponsoring

Thank you for considering sponsoring! While this project does not require
sponsoring, small donations are accepted. 100% of the donations are used to
provide a student (me) :man_student: with a steady supply of caffeinated beverages
which are then metabolized into 100% organic Rust code.

## License

This project is licensed under the MIT license, which you find
[here](https://github.com/HiImJulien/in-vite/blob/master/LICENSE.md).


