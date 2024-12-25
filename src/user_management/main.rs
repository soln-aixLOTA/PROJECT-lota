use crate::models::User;

async fn create_user(user: web::Json<User>) -> impl Responder {
    if let Err(e) = user.validate() {
        return HttpResponse::BadRequest().json(e); // Return validation errors
    }
    // ... proceed with user creation ...
} 