use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::{
    domain::{StudyTopic, StudyTopicInfo, Subject},
    study_service::{StudyService, StudySessionResponse},
};

#[derive(Clone)]
struct ApiState {
    study_service: StudyService,
}

pub async fn start_api(study_service: StudyService, port: String) {
    let state = ApiState { study_service };

    let cors = CorsLayer::very_permissive();

    let app = Router::new()
        .route("/", get(health_check))
        .route("/study_topics", get(get_study_topics))
        .route("/study_topic", post(add_study_topic))
        .route("/study_topic/{study_topic_id}", delete(delete_study_topic))
        .route("/study_topics_today", get(get_study_topics_today))
        .route("/subjects", get(get_subjects))
        .route(
            "/study_topic/subject/{subject_name}",
            get(get_study_topics_for_subject),
        )
        .route(
            "/study_session/{subject_name}",
            get(get_study_sessions_for_subject),
        )
        .route("/subject/{subject_name}", post(add_subject))
        .route("/subject/{subject_name}", delete(delete_subject))
        .route(
            "/study_session/complete/{study_session_id}",
            post(complete_study_session),
        )
        .layer(cors)
        .with_state(state);

    info!("Trying to run in port: {port}");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("Failed to bind to port");
    info!("Running in port: {port}");

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "I am alive"
}

async fn complete_study_session(
    State(state): State<ApiState>,
    Path(study_session_id): Path<i64>,
) -> StatusCode {
    match state
        .study_service
        .complete_study_session(study_session_id)
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            error!("Error completing study session: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn delete_subject(
    State(state): State<ApiState>,
    Path(subject_name): Path<String>,
) -> StatusCode {
    match state.study_service.delete_subject(subject_name).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            error!("Error deleting subject: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn add_subject(
    State(state): State<ApiState>,
    Path(subject_name): Path<String>,
) -> StatusCode {
    match state.study_service.add_subject(subject_name).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            error!("Error adding subject: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn get_subjects(State(state): State<ApiState>) -> (StatusCode, Json<Vec<Subject>>) {
    match state.study_service.get_study_subjects().await {
        Ok(subjects) => (StatusCode::OK, Json(subjects)),
        Err(err) => {
            error!("Error in get subjects function: {err}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

async fn get_study_topics_for_subject(
    State(state): State<ApiState>,
    Path(subject_name): Path<String>,
) -> (StatusCode, Json<Vec<StudyTopic>>) {
    match state
        .study_service
        .get_study_topics_for_subject(subject_name)
        .await
    {
        Ok(study_topics) => (StatusCode::OK, Json(study_topics)),
        Err(err) => {
            error!("Error getting study topic for subject: {err}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

async fn get_study_sessions_for_subject(
    State(state): State<ApiState>,
    Path(subject_name): Path<String>,
) -> (StatusCode, Json<Vec<StudySessionResponse>>) {
    match state
        .study_service
        .get_study_sessions_for_subject(subject_name)
        .await
    {
        Ok(study_sessions) => (StatusCode::OK, Json(study_sessions)),
        Err(err) => {
            error!("Error getting study session for subject: {err}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
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
