use actix_cors::Cors;
use actix_http::error::Error;
use actix_web::middleware::Logger;
use actix_web::web::{Data, Json};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use juniper::http::playground::playground_source;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;

use warpgrapher::{Engine};
use warpgrapher::engine::config::{Configuration};
use warpgrapher::engine::database::neo4j::Neo4jEndpoint;
use warpgrapher::engine::database::DatabaseEndpoint;
use warpgrapher::juniper::http::GraphQLRequest;

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
    let engine = &data.engine;
    let metadata: HashMap<String, String> = HashMap::new();
    let resp = engine.execute(&req.into_inner(), &metadata).await;
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

async fn create_engine(config: Configuration) -> Engine<()> {
    let db = Neo4jEndpoint::from_env()
        .expect("Failed to parse endpoint from environment")
        .pool()
        .await
        .expect("Failed to create db endpoint");
    let engine: Engine<()> = Engine::new(config, db)
        .build()
        .expect("Failed to build engine");
    engine
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = clap::App::new("warpgrapher-actixweb")
        .version("0.6.0")
        .about("Warpgrapher sample application using actix-web server")
        .author("Warpgrapher")
        .arg(
            clap::Arg::with_name("CONFIG")
                .help("Path to configuration file to use")
                .required(true),
        )
        .get_matches();

    let cfn = matches.value_of("CONFIG").expect("Configuration required.");

    let config_file = File::open(cfn.to_string())
          .expect("Could not read file");
    let config = Configuration::try_from(config_file)
          .expect("Failed to parse config file");

    let engine = create_engine(config.clone()).await;

    let graphql_endpoint = "/graphql";
    let playground_endpoint = "/graphiql";
    let bind_addr = "127.0.0.1".to_string();
    let bind_port = "5000".to_string();
    let addr = format!("{}:{}", bind_addr, bind_port);

    let app_data = AppData::new(engine);

    println!("Starting server on {}", addr);
    HttpServer::new(move || {
        App::new()
            .data(app_data.clone())
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .route(graphql_endpoint, web::post().to(graphql))
            .route(playground_endpoint, web::get().to(graphiql))
    })
    .bind(&addr)
    .expect("Failed to start server")
    .run()
    .await
}
