use actix_cors::Cors;
use actix_http::error::Error;
use actix_web::middleware::Logger;
use actix_web::web::{Data, Json};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;

use warpgrapher::engine::config::Configuration;
use warpgrapher::engine::context::RequestContext;
use warpgrapher::engine::database::cypher::CypherEndpoint;
use warpgrapher::engine::database::DatabaseEndpoint;
use warpgrapher::juniper::http::playground::playground_source;
use warpgrapher::Engine;

#[derive(Clone)]
struct AppData {
    engine: Engine<Rctx>,
}

impl AppData {
    fn new(engine: Engine<Rctx>) -> AppData {
        AppData { engine }
    }
}

#[derive(Clone, Debug)]
struct Rctx {}

impl RequestContext for Rctx {
    type DBEndpointType = CypherEndpoint;

    fn new() -> Self {
        Rctx {}
    }
}

#[derive(Clone, Debug, Deserialize)]
struct GraphqlRequest {
    pub query: String,
    pub variables: Option<Value>,
}

async fn graphql(data: Data<AppData>, req: Json<GraphqlRequest>) -> Result<HttpResponse, Error> {
    let engine = &data.engine;
    let metadata: HashMap<String, String> = HashMap::new();
    let resp = engine
        .execute(req.query.to_string(), req.variables.clone(), metadata)
        .await;
    match resp {
        Ok(body) => Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body.to_string())),
        Err(e) => Ok(HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(e.to_string())),
    }
}

async fn playground(_data: Data<AppData>) -> impl Responder {
    let html = playground_source("/graphql", None);
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn create_engine(config: Configuration) -> Engine<Rctx> {
    let db = CypherEndpoint::from_env()
        .expect("Failed to parse endpoint from environment")
        .pool()
        .await
        .expect("Failed to create db endpoint");
    let engine: Engine<Rctx> = Engine::<Rctx>::new(config, db)
        .build()
        .expect("Failed to build engine");
    engine
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = clap::App::new("warpgrapher-actixweb")
        .version("0.9.0")
        .about("Warpgrapher sample application using actix-web server")
        .author("Warpgrapher")
        .arg(
            clap::Arg::new("CONFIG")
                .help("Path to configuration file to use")
                .required(true),
        )
        .get_matches();

    let cfn = matches.value_of("CONFIG").expect("Configuration required.");

    let config_file = File::open(cfn.to_string()).expect("Could not read file");
    let config = Configuration::try_from(config_file).expect("Failed to parse config file");

    let engine = create_engine(config.clone()).await;

    let graphql_endpoint = "/graphql";
    let playground_endpoint = "/playground";
    let bind_addr = "0.0.0.0".to_string();
    let bind_port = "5000".to_string();
    let addr = format!("{}:{}", bind_addr, bind_port);

    let app_data = AppData::new(engine);

    println!("Starting server on {}", addr);
    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(app_data.clone()))
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .route(graphql_endpoint, web::post().to(graphql))
            .route(playground_endpoint, web::get().to(playground))
    })
    .bind(&addr)
    .expect("Failed to start server")
    .run()
    .await
}
