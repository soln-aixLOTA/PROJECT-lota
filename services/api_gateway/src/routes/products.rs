use actix_web::{web, HttpResponse};
use uuid::Uuid;
use validator::Validate;

use crate::{
    db::{self, DbPool},
    error::ApiError,
    models::{ApiResponse, CreateProductRequest},
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::post().to(create_product))
        .route("", web::get().to(list_products))
        .route("/{id}", web::get().to(get_product))
        .route("/{id}", web::put().to(update_product))
        .route("/{id}", web::delete().to(delete_product));
}

async fn create_product(
    pool: web::Data<DbPool>,
    product_data: web::Json<CreateProductRequest>,
) -> Result<HttpResponse, ApiError> {
    // Validate request
    product_data.validate()?;

    // Create product in database
    let product_id = db::create_product(
        &pool,
        &product_data.name,
        product_data.description.as_deref(),
        product_data.price.clone(),
        product_data.stock,
    )
    .await?;

    // Fetch created product
    let product = db::get_product_by_id(&pool, product_id)
        .await?
        .ok_or_else(|| ApiError::InternalError("Failed to fetch created product".to_string()))?;

    Ok(HttpResponse::Created().json(ApiResponse::success(product)))
}

async fn list_products(pool: web::Data<DbPool>) -> Result<HttpResponse, ApiError> {
    let products = db::list_products(&pool, 100, 0).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(products)))
}

async fn get_product(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let id = id.into_inner();
    let product = db::get_product_by_id(&pool, id)
        .await?
        .ok_or_else(|| ApiError::NotFoundError("Product not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(product)))
}

async fn update_product(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
    product_data: web::Json<CreateProductRequest>,
) -> Result<HttpResponse, ApiError> {
    // Validate request
    product_data.validate()?;

    let id = id.into_inner();
    // Update product in database
    let updated = db::update_product(
        &pool,
        id,
        &product_data.name,
        product_data.description.as_deref(),
        product_data.price.clone(),
        product_data.stock,
    )
    .await?;

    if !updated {
        return Err(ApiError::NotFoundError("Product not found".to_string()));
    }

    // Fetch updated product
    let product = db::get_product_by_id(&pool, id)
        .await?
        .ok_or_else(|| ApiError::InternalError("Failed to fetch updated product".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(product)))
}

async fn delete_product(
    pool: web::Data<DbPool>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let deleted = db::delete_product(&pool, id.into_inner()).await?;

    if !deleted {
        return Err(ApiError::NotFoundError("Product not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}
