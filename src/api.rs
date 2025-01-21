use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::{
    domain::{StudyTopic, StudyTopicInfo},
    study_service::StudyService,
};

#[derive(Clone)]
struct ApiState {
    study_service: StudyService,
}

pub async fn start_api(study_service: StudyService) {
    let state = ApiState { study_service };

    let cors = CorsLayer::very_permissive();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/study_topics", get(get_study_topics))
        .route("/study_topic", post(add_study_topic))
        .route("/study_topic/{study_topic_id}", delete(delete_study_topic))
        .route("/study_topics_today", get(get_study_topics_today))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to port 8080");
    info!("Running in port: 8080");

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "I am alive"
}

async fn get_study_topics_today(
    State(state): State<ApiState>,
) -> (StatusCode, Json<Vec<StudyTopic>>) {
    match state.study_service.get_study_topics_for_today().await {
        Ok(study_topics_today) => (StatusCode::OK, Json(study_topics_today)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new())),
    }
}

async fn get_study_topics(State(state): State<ApiState>) -> (StatusCode, Json<Vec<StudyTopic>>) {
    match state.study_service.get_study_topics().await {
        Ok(study_topics) => (StatusCode::OK, Json(study_topics)),
        Err(err) => {
            error!("Error: {err}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

async fn delete_study_topic(
    State(state): State<ApiState>,
    Path(study_topic_id): Path<i64>,
) -> StatusCode {
    match state.study_service.delete_study_topic(study_topic_id).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            error!("Error adding study topic with error: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn add_study_topic(
    State(state): State<ApiState>,
    Json(body): Json<StudyTopicInfo>,
) -> StatusCode {
    match state.study_service.add_study_topic(body).await {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            error!("Error adding study topic with error: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
