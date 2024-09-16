use axum::{routing::get, Router, response::Html, extract::State};
use tera::{Tera, Context};
use in_vite::Vite;

async fn hello(State(state): State<AppState>) -> Html<String> {
    let tera = &state.tera;
    let html = tera.render("index.html", &Context::new()).expect("");

    Html(html)
}

#[derive(Clone)]
struct AppState {
    tera: Tera,
}

#[tokio::main]
async fn main() {
    let vite = Vite::default();
    let mut tera = tera::Tera::new("src/templates/**/*.html").expect("templates should be loadable");
    tera.register_function("vite", vite);

    let state = AppState { tera };

    let app = Router::new()
        .route("/", get(hello))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
