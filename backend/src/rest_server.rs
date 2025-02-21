//! REST API component for exposing queryable endpoints both for a REST API client
//! user and the fronted part of application for features like message history.

use std::net::TcpListener;

use actix_web::{get, http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};

use crate::SharedServerState;

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/history")]
async fn get_history(server_state: web::Data<SharedServerState>) -> impl Responder {
    let history = &server_state.get_ref().lock().await.history;
    log::trace!("history is queried: '/history'");
    let j = serde_json::to_string(&history).unwrap();
    HttpResponse::Ok().content_type(ContentType::json()).body(j)
}

/// Entry for starting REST API server.
pub async fn run_rest_server(listener: TcpListener, server_state: SharedServerState) {
    HttpServer::new(move || {
        let web_data = web::Data::new(server_state.clone());
        App::new()
            .service(health)
            .service(get_history)
            .app_data(web_data)
    })
    .listen(listener)
    .expect("failed to start REST API server")
    .run()
    .await
    .expect("unable to run REST API server");
}
