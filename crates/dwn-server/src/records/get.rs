use axum::extract::{Path, State};
use dwn::Dwn;

pub async fn records_get(
    Path((_target, _record_id)): Path<(String, String)>,
    State(_dwn): State<Dwn>,
) {
    todo!();
}
