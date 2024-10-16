use axum::extract::{Path, State};
use dwn::DWN;

pub async fn records_get(
    Path((_target, _record_id)): Path<(String, String)>,
    State(_dwn): State<DWN>,
) {
    todo!();
}
