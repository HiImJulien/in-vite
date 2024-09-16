use std::path::PathBuf;

use actix_web::{web, App, HttpServer, get, Responder};
use minijinja::{Environment, Value};
use minijinja::path_loader;
use minijinja_autoreload::AutoReloader;

use in_vite::Vite;

#[get("/")]
async fn index(loader: web::Data<minijinja_autoreload::AutoReloader>) -> impl Responder {
    loader
        .acquire_env()
        .unwrap()
        .get_template("index.html")
        .unwrap()
        .render(Value::UNDEFINED)
        .map(web::Html::new)
        .unwrap()
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let vite = Vite::default();
    let mut env = Environment::new();
    env.add_global("vite", Value::from_object(vite));

    let loader = AutoReloader::new(move |notif| {
        let mut env: Environment<'static> = Environment::new();
        let path = PathBuf::from("src").join("templates");

        notif.watch_path(&path, true);

        env.set_loader(path_loader(path));

        let vite = Vite::default();
        env.add_global("vite", Value::from_object(vite));

        Ok(env)
    });

    let loader = web::Data::new(loader);

    HttpServer::new(move || App::new()
        .app_data(loader.clone())
        .service(index)
    )
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
