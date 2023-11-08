#![allow(dead_code, unused_imports)]

use axum::{
    body::Body,
    body::Bytes,
    extract::{self, Query},
    http::{header, HeaderMap, Request, StatusCode, Uri},
    middleware::{self, Next},
    response::Response,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use minijinja::{context, value::Object, Environment, Template};
use serde_json::Value;
use std::{collections::HashMap, fmt::format, fs::FileType};
use urlencoding::decode;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        // -- HTMX
        .route("/htmx", get(htmx))
        // -- PAGES
        .route("/", get(index))
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
        .route("/tst", get(|| async { "{\"kill\": 7}" }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn tyler_post(x: String) -> Html<String> {
    let x = decode(&x["_=".len()..]).unwrap().to_string();
    let data: Value = serde_json::from_str(&x).unwrap();
    //let Value::Object(data) = data else { panic!() };

    Html(format!(
        "<pre>{:#?}</pre>",
        data.as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.to_string(), val_type(v)))
            .collect::<HashMap<String, String>>()
    ))
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

async fn htmx() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        include_str!("htmx.min.js"),
    )
}

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
