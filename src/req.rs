use serde::{de::DeserializeOwned, Serialize};

use crate::{consts::TOKEN, err, Result};

pub async fn post<S: Serialize, D: DeserializeOwned>(uri: &str, body: S) -> Result<D> {
    match awc::Client::new()
        .post(uri)
        .append_header(("Authorization", TOKEN))
        .send_body(
            serde_json::to_vec(&body)
                .map_err(|_| format!("Failed to serialise request to {uri}"))?,
        )
        .await
    {
        Ok(mut resp) => Ok(resp
            .json::<D>()
            .await
            .map_err(|e| format!("Failed to deserialise response: {e}"))?),
        Err(e) => err(e),
    }
}

pub fn uri(api_path: &str) -> String {
    const API_URL: &str = "https://discord.com/api/v10";
    format!("{API_URL}{api_path}")
}
