use axum::{routing::get, Router};

use crate::study_service::StudyService;

#[derive(Clone)]
struct ApiState {
    study_service: StudyService,
}

pub async fn start_api(study_service: StudyService) {
    let state = ApiState { study_service };

    let app = Router::new()
        .with_state(state)
        .route("/", get(health_check));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "I am alive"
}
