use actix_web::{HttpResponse, Responder};

pub(crate) async fn readiness_check() -> impl Responder {
    HttpResponse::Ok().finish()
}
