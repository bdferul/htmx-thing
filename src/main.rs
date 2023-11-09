#![allow(dead_code, unused_imports)]

use axum::{
    body::{Body, Bytes},
    extract::{self, Path, Query},
    http::{header, HeaderMap, Request, StatusCode, Uri},
    middleware::{self, Next},
    response::Response,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, Router,
};
use minijinja::{context, value::Object, Environment, Template};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::{format, Debug},
    fs::{self, FileType},
    os::unix::prelude::OsStrExt,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use urlencoding::decode;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/assets/js/:file_name", get(static_js_file))
        // -- PAGES
        .route("/", get(index))
        .route(
            "/form",
            get(|| async { MyEnvironment::get_html("form.html") }).post(form_post),
        )
        .route(
            "/tyler",
            get(|| async { MyEnvironment::get_html("tyler.html") }).post(tyler_post),
        )
        .route(
            "/mouse",
            get(|| async { MyEnvironment::get_html("mouse.html") }), //impl Fn() -> impl Future<Output = Html<String>>
        )
        // -- API
        .route("/mouse_entered", post(|| async { println!("mouse enter") }))
        .route("/tst", get(|| async { "{\"kill\": 7}" }))
        .layer(middleware::from_fn(log_layer));

    println!("Running on http://localhost:3000");

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn static_js_file(Path(file_name): Path<String>) -> impl IntoResponse {
    if !dir_child_names("assets/js").contains(&file_name) {
        panic!()
    }
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        fs::read_to_string(format!("assets/js/{file_name}")).unwrap(),
    )
}

fn dir_child_names(path: impl AsRef<std::path::Path>) -> Vec<String> {
    fs::read_dir(path)
        .unwrap()
        .map(|x| x.unwrap().file_name().to_str().unwrap().to_string())
        .collect()
}

async fn form_post(Form(x): Form<Value>) -> String {
    println!("{x}");
    "success".to_string()
}

#[derive(Serialize, Deserialize, Debug)]
struct Pokemon {
    name: String,
    id: usize,
    height: usize,
    weight: usize,
    #[serde(rename = "game_indices")]
    games: Vec<PV>,
}

#[derive(Serialize, Deserialize)]
struct PV {
    version: PVS,
}

impl Debug for PV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version.name)
    }
}

#[derive(Serialize, Deserialize)]
struct PVS {
    name: String,
}

async fn tyler_post(Form(x): Form<Value>) -> Html<String> {
    println!("tyler");
    let x = x["json"].as_str().unwrap_or("{'error':'no data'}");
    let data: Pokemon = serde_json::from_str(x).unwrap();
    //let Value::Object(data) = data else { panic!() };

    Html(format!("<pre>{data:#?}</pre>",))
}

fn val_type(v: &Value) -> String {
    use Value as E;
    match v {
        E::Null => "null",
        E::Array(_) => "array",
        E::Bool(_) => "bool",
        E::Number(_) => "number",
        E::Object(_) => "object",
        E::String(_) => "string",
    }
    .to_string()
}

async fn log_layer(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    println!(
        "{} {} {}",
        req.method(),
        req.uri(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    );
    let res = next.run(req).await;

    println!("{}", res.status());
    Ok(res)
}

async fn htmx() -> impl IntoResponse {}

async fn index() -> Html<String> {
    let env = MyEnvironment::default().0;
    let tmpl = env.get_template("index.html").unwrap();

    Html(tmpl.render(context! {}).unwrap())
}
async fn mouse() -> Html<String> {
    let env = MyEnvironment::default().0;
    let tmpl = env.get_template("mouse.html").unwrap();

    Html(tmpl.render(context! {}).unwrap())
}

struct MyEnvironment<'a>(Environment<'a>);

impl<'a> MyEnvironment<'a> {
    fn get_html(name: &str) -> Html<String> {
        println!("JINJA: {name}");
        Html(
            Self::default()
                .0
                .get_template(name)
                .unwrap()
                .render(context! {})
                .unwrap(),
        )
    }
}

impl<'a> Default for MyEnvironment<'a> {
    fn default() -> Self {
        let mut env = Environment::new();
        for file in std::fs::read_dir("templates").unwrap() {
            let file = file.unwrap();
            if file.file_type().unwrap().is_dir() {
                continue;
            }
            let Ok(template) = std::fs::read_to_string(file.path()) else {
                continue;
            };

            env.add_template_owned(
                file.path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                template,
            )
            .unwrap()
        }

        Self(env)
    }
}
