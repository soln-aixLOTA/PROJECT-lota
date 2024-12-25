use actix_web::{get, post, HttpResponse, Responder};

#[get("/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}

#[post("/login")]
pub async fn login() -> impl Responder {
    HttpResponse::Ok().body("Logged in")
}

#[get("/users")]
pub async fn get_users() -> impl Responder {
    HttpResponse::Ok().body("GET users")
}

#[post("/users")]
pub async fn create_user() -> impl Responder {
    HttpResponse::Ok().body("Created user")
}

#[get("/users/{id}")]
pub async fn get_user() -> impl Responder {
    HttpResponse::Ok().body("GET user by ID")
}
