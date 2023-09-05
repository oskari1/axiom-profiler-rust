use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use prototype::parsers::{z3parser1, LogParser};
use actix_cors::Cors;
const SIZE_LIMIT: usize = 1 << 32;

// example endpoint
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

// example endpoint
#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[post("/parse")]
async fn parse_file(req_body: String) -> impl Responder {
    use std::fs;
    if fs::write("temp.log", req_body).is_err() {
        return HttpResponse::InternalServerError().body("Error writing temp file");
    }
    let mut parser = z3parser1::Z3Parser1::new();
    let settings = prototype::file_io::Settings {
                save_all_data: true,
                reuses: false,
                verbose: false,
                file: "temp.log".to_string(),
                sort: "cost".to_string(),
                timeout: 0.0,
                line_limit: usize::MAX
            };
    let result = parser.process_file("temp.log", &settings);
    let result = 
    match result {
        Ok(r) => r.0,
        Err(r) => r
    };
    HttpResponse::Ok().body(result)
}

#[get("/sample")]
async fn sample_prototype_call() -> impl Responder {
    let mut parser = z3parser1::Z3Parser1::new();
    let settings = prototype::file_io::Settings {
                save_all_data: true,
                reuses: false,
                verbose: false,
                file: "logs/heaps-simpler4.log".to_string(),
                sort: "cost".to_string(),
                timeout: 0.0,
                line_limit: usize::MAX
            };
    let result = parser.process_file("logs/heaps-simpler4.log", &settings);
    let result = 
    match result {
        Ok(r) => r.0,
        Err(r) => r
    };
    HttpResponse::Ok().body(result)
}

/// Actix server to handle requests from frontend (e.g. Yew)
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::permissive();
        let config = web::PayloadConfig::new(SIZE_LIMIT);
        App::new()
        .wrap(cors)
            .service(hello)
            .service(echo)
            .service(sample_prototype_call)
            .service(parse_file)
            .app_data(config)
    })
    .bind(("127.0.0.1", 1234))?
    .run()
    .await
}