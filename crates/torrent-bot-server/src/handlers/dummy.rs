use actix_web::{HttpResponse, Responder};

pub(crate) async fn dummy() -> impl Responder {
    HttpResponse::Ok().body("It works!")
}
