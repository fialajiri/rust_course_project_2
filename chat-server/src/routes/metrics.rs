use std::sync::Arc;

use rocket::get;
use rocket::State;

use crate::utils::metrics::Metrics;

#[get("/metrics")]
pub async fn get_metrics(metrics: &State<Arc<tokio::sync::Mutex<Metrics>>>) -> String {
    let metrics = metrics.lock().await;
    metrics.get_metrics()
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![get_metrics]
}
