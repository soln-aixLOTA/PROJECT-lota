use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use uuid::Uuid;

use crate::config::Config;
use crate::errors::Error;
use crate::models::{
    BatchInferenceRequest, BatchInferenceResponse, InferenceRequest, InferenceResponse,
};
use crate::repositories::InferenceRepository;
use crate::services::InferenceService;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inference")
            .route("", web::post().to(run_inference))
            .route("/batch", web::post().to(run_batch_inference))
            .route("/{id}", web::get().to(get_inference))
            .route("/agent/{agent_id}", web::get().to(get_agent_inferences)),
    );
}

async fn run_inference(
    repo: web::Data<InferenceRepository>,
    inference_service: web::Data<InferenceService>,
    request: web::Json<InferenceRequest>,
) -> Result<impl Responder, Error> {
    // Run inference
    let (output, confidence_score) = inference_service.run(&request).await?;

    // Store result
    let record = repo.create(&request, output, confidence_score).await?;

    Ok(HttpResponse::Created().json(InferenceResponse {
        id: record.id,
        agent_id: record.agent_id,
        model_id: record.model_id,
        input: record.input,
        output: record.output,
        confidence_score: record.confidence_score,
        metadata: record.metadata,
        created_at: record.created_at,
    }))
}

async fn run_batch_inference(
    repo: web::Data<InferenceRepository>,
    inference_service: web::Data<InferenceService>,
    request: web::Json<BatchInferenceRequest>,
) -> Result<impl Responder, Error> {
    let mut results = Vec::new();

    // Process each input
    for input in &request.inputs {
        let single_request = InferenceRequest {
            agent_id: request.agent_id,
            model_id: request.model_id.clone(),
            input: input.clone(),
            metadata: request.metadata.clone(),
        };

        let (output, confidence_score) = inference_service.run(&single_request).await?;
        let record = repo
            .create(&single_request, output, confidence_score)
            .await?;

        results.push(InferenceResponse {
            id: record.id,
            agent_id: record.agent_id,
            model_id: record.model_id,
            input: record.input,
            output: record.output,
            confidence_score: record.confidence_score,
            metadata: record.metadata,
            created_at: record.created_at,
        });
    }

    Ok(HttpResponse::Created().json(BatchInferenceResponse {
        results,
        metadata: json!({
            "total_processed": results.len(),
            "batch_id": Uuid::new_v4(),
        }),
    }))
}

async fn get_inference(
    repo: web::Data<InferenceRepository>,
    id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let record = repo
        .find_by_id(id.into_inner())
        .await?
        .ok_or_else(|| Error::NotFound("Inference record not found".into()))?;

    Ok(HttpResponse::Ok().json(InferenceResponse {
        id: record.id,
        agent_id: record.agent_id,
        model_id: record.model_id,
        input: record.input,
        output: record.output,
        confidence_score: record.confidence_score,
        metadata: record.metadata,
        created_at: record.created_at,
    }))
}

async fn get_agent_inferences(
    repo: web::Data<InferenceRepository>,
    agent_id: web::Path<Uuid>,
) -> Result<impl Responder, Error> {
    let records = repo.find_by_agent(agent_id.into_inner()).await?;

    let responses: Vec<InferenceResponse> = records
        .into_iter()
        .map(|record| InferenceResponse {
            id: record.id,
            agent_id: record.agent_id,
            model_id: record.model_id,
            input: record.input,
            output: record.output,
            confidence_score: record.confidence_score,
            metadata: record.metadata,
            created_at: record.created_at,
        })
        .collect();

    Ok(HttpResponse::Ok().json(responses))
}
