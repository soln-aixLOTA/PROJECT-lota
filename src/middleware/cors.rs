use actix_cors::Cors;
use actix_web::http::header;

pub fn configure_cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
        .expose_headers(&[
            header::CONTENT_DISPOSITION,
            header::CONTENT_TYPE,
            header::CONTENT_LENGTH,
        ])
        .supports_credentials()
}
