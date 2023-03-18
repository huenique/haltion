use axum::http::StatusCode;
use reqwest::Client;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Tenant {
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TenantResult {
    pub detail: String,
    pub status: StatusCode,
}

pub async fn create_tenant(
    client: &Client,
    db_url: &String,
    auth: &String,
    tenant: &String,
) -> TenantResult {
    let resp = match client
        .post(db_url)
        .header("Accept", "application/json")
        .header("Authorization", auth)
        .header("DB", tenant)
        .header("NS", tenant)
        .body(format!("DEFINE NAMESPACE {tenant}"))
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            return TenantResult {
                detail: err.to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR,
            };
        }
    };

    return TenantResult {
        status: resp.status(),
        detail: match resp.json::<String>().await {
            Ok(json) => json,
            Err(err) => err.to_string(),
        },
    };
}
