use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::body::BoxBody;
use actix_web_lab::sse::{Event, Sse};
use actix_web_lab::__reexports::futures_util::stream::BoxStream;
use std::convert::Infallible;

pub enum EitherResponder {
    HttpResponse(HttpResponse),
    Sse(Sse<BoxStream<'static, Result<Event, Infallible>>>),
}

impl Responder for EitherResponder {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            Self::HttpResponse(res) => res.respond_to(req),
            Self::Sse(stream) => stream.respond_to(req),
        }
    }
}