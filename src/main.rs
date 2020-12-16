use actix::System;
use actix_cors::Cors;
use actix_http::error::Error;
use actix_web::middleware::Logger;
use actix_web::web::{Data, Json};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use juniper::http::playground::playground_source;
use std::collections::HashMap;
use std::convert::TryFrom;

use warpgrapher::{Engine};
use warpgrapher::engine::config::{Configuration};
use warpgrapher::engine::database::neo4j::Neo4jEndpoint;
use warpgrapher::engine::database::DatabaseEndpoint;
use warpgrapher::juniper::http::GraphQLRequest;

static CONFIG: &'static str = "
version: 1
model:
  - name: User
    props:
      - name: name
        type: String
  - name: Project
    props:
      - name: name
        type: String
    rels:
      - name: users
        nodes: [User]
        list: true
";

#[derive(Clone)]
struct AppData {
    engine: Engine,
}

impl AppData {
    fn new(engine: Engine) -> AppData {
        AppData { engine }
    }
}

async fn graphql(data: Data<AppData>, req: Json<GraphQLRequest>) -> Result<HttpResponse, Error> {
    let metadata: HashMap<String, String> = HashMap::new();

    let resp = &data.engine.execute(&req.into_inner(), &metadata);

    match resp {
        Ok(body) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body.to_string())),
        Err(e) => Ok(HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(e.to_string())),
    }
}

async fn graphiql(_data: Data<AppData>) -> impl Responder {
    let html = playground_source(&"/graphql", None);

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn create_engine() -> Engine<(), ()> {
    // parse warpgrapher config
    let config = Configuration::try_from(CONFIG.to_string())
        .expect("Failed to parse CONFIG");

    // define database endpoint
    let db = Neo4jEndpoint::from_env()
        .expect("Failed to parse endpoint from environment")
        .pool().await
        .expect("Failed to create database pool");

    // create warpgrapher engine
    let engine: Engine<(), ()> = Engine::new(config, db)
        .build()
        .expect("Failed to build engine");

    engine
}

#[tokio::main]
async fn main() {
    clap::App::new("warpgrapher-actixweb")
        .version("0.1")
        .about("Warpgrapher sample application using actix-web server")
        .author("Warpgrapher");

    let engine = create_engine().await;

    let graphql_endpoint = "/graphql";
    let playground_endpoint = "/graphiql";
    let bind_addr = "127.0.0.1".to_string();
    let bind_port = "5000".to_string();
    let addr = format!("{}:{}", bind_addr, bind_port);

    let sys = System::new("warpgrapher-actixweb");

    let app_data = AppData::new(engine);

    HttpServer::new(move || {
        App::new()
            .data(app_data.clone())
            .wrap(Logger::default())
            .wrap(Cors::default())
            .route(graphql_endpoint, web::post().to(graphql))
            .route(playground_endpoint, web::get().to(graphiql))
    })
    .bind(&addr)
    .expect("Failed to start server")
    .run();

    println!("Server available on: {:#?}", &addr);
    let _ = sys.run();
}
