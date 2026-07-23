use axum::extract::State;
use axum::Json;
use serde::Serialize;

use super::router::AppState;
use crate::plugins::traits::BrandingPayload;

/// Response for a single provider.
#[derive(Serialize)]
pub struct ProviderResponse {
    pub id: String,
    #[serde(flatten)]
    pub branding: BrandingPayload,
}

/// Response for GET /api/v1/providers.
#[derive(Serialize)]
pub struct ProvidersResponse {
    pub providers: Vec<ProviderResponse>,
}

/// K-035: GET /api/v1/providers — Returns branding info for all loaded plugins.
pub async fn list_providers(
    State(state): State<AppState>,
) -> Json<ProvidersResponse> {
    let manager = state.plugin_manager.read().await;

    let providers = manager
        .plugins()
        .iter()
        .map(|p| ProviderResponse {
            id: p.id.clone(),
            branding: p.branding.clone(),
        })
        .collect();

    Json(ProvidersResponse { providers })
}
