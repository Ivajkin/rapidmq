use actix_web::{web, App, HttpResponse, HttpServer, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{RapidMQ, Message};
use bcrypt::{hash, verify};
use prometheus::{Encoder, TextEncoder};
use actix_files::Files;

#[derive(Deserialize)]
struct PublishRequest {
    queue_name: String,
    message: String,
}

#[derive(Serialize)]
struct MessageResponse {
    id: String,
    content: String,
}

#[derive(Deserialize)]
struct Credentials {
    username: String,
    password: String,
}

async fn authenticate(req: HttpRequest, credentials: web::Json<Credentials>) -> impl Responder {
    // In a real-world scenario, you would check against a database
    let hashed_password = hash(&credentials.password, 4).unwrap();
    if credentials.username == "admin" && verify(&credentials.password, &hashed_password).unwrap() {
        // In a real-world scenario, you would generate and store a session token
        HttpResponse::Ok().body("Authenticated")
    } else {
        HttpResponse::Unauthorized().body("Invalid credentials")
    }
}

async fn create_queue(
    req: HttpRequest,
    rapidmq: web::Data<RapidMQ>,
    queue_name: web::Path<String>,
) -> impl Responder {
    if !is_authenticated(&req) {
        return HttpResponse::Unauthorized().body("Authentication required");
    }
    rapidmq.create_queue(&queue_name);
    HttpResponse::Ok().body(format!("Queue '{}' created", queue_name))
}

async fn publish_message(
    req: HttpRequest,
    rapidmq: web::Data<RapidMQ>,
    req_body: web::Json<PublishRequest>,
) -> impl Responder {
    if !is_authenticated(&req) {
        return HttpResponse::Unauthorized().body("Authentication required");
    }
    let message = Message {
        id: Uuid::new_v4().to_string(),
        content: req_body.message.clone(),
    };
    rapidmq.publish(&req_body.queue_name, message).await;
    HttpResponse::Ok().body("Message published")
}

async fn consume_message(
    req: HttpRequest,
    rapidmq: web::Data<RapidMQ>,
    queue_name: web::Path<String>,
) -> impl Responder {
    if !is_authenticated(&req) {
        return HttpResponse::Unauthorized().body("Authentication required");
    }
    if let Some(message) = rapidmq.consume(&queue_name).await {
        let response = MessageResponse {
            id: message.id,
            content: message.content,
        };
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::NotFound().body("No messages in queue")
    }
}

fn is_authenticated(req: &HttpRequest) -> bool {
    // In a real-world scenario, you would validate the session token
    req.headers().contains_key("Authorization")
}

async fn metrics() -> impl Responder {
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&metrics::REGISTRY.gather(), &mut buffer).unwrap();
    HttpResponse::Ok().body(String::from_utf8(buffer).unwrap())
}

pub async fn start_api(rapidmq: RapidMQ) -> std::io::Result<()> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(rapidmq.clone()))
            .service(Files::new("/dashboard", "static").index_file("dashboard.html"))
            .route("/authenticate", web::POST().to(authenticate))
            .route("/queue/{name}", web::POST().to(create_queue))
            .route("/publish", web::POST().to(publish_message))
            .route("/consume/{queue_name}", web::GET().to(consume_message))
            .route("/metrics", web::GET().to(metrics))
    })
    .bind_openssl("127.0.0.1:8080", builder)?
    .run()
    .await
}