use serde::{de::DeserializeOwned, Serialize};

use crate::{consts::TOKEN, discord::ErrorResponse, err, Result};

pub async fn post<S: Serialize, D: DeserializeOwned>(uri: &str, body: S) -> Result<D> {
    let body =
        serde_json::to_vec(&body).map_err(|_| format!("Failed to serialise request to {uri}"))?;

    let res = awc::Client::new()
        .post(uri)
        .insert_header(("Authorization", TOKEN))
        .content_type("application/json")
        .send_body(body)
        .await;

    let bytes = match res {
        Ok(mut resp) => match resp.body().await {
            Ok(bytes) => bytes,
            Err(e) => return err(e),
        },
        Err(e) => return err(e),
    };

    if let Ok(resp) = serde_json::from_slice::<D>(&bytes) {
        Ok(resp)
    } else {
        match serde_json::from_slice::<ErrorResponse>(&bytes) {
            Ok(err_resp) => Err(err_resp.message),
            Err(e) => Err(format!("Failed to deserialise response: {e}")),
        }
    }
}

pub fn uri(api_path: &str) -> String {
    const API_URL: &str = "https://discord.com/api/v10";
    format!("{API_URL}{api_path}")
}
