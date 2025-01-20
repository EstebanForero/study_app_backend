use libsql::{Builder, Database};

struct Repository {
    db: Database,
}

impl Repository {
    async fn new(url: String, token: String) -> Result<(), String> {
        let db = Builder::new_remote(url, token)
            .build()
            .await
            .map_err(|err| format!("Error creating new remote daabase for libsql: {err}"))?;
    }
}
