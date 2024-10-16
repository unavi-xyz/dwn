use axum::extract::Path;
use tracing::info;

pub async fn records_get(Path(id): Path<String>) {
    info!("GET records/{}", id);
}
