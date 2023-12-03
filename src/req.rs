use std::fmt::Display;

use serde::{de::DeserializeOwned, Serialize};

use crate::{consts::TOKEN, discord::ErrorResponse, err, msg, Result};

pub async fn get<U: AsRef<str>>(uri: U) -> Result<String> {
    reqwest::get(uri.as_ref())
        .await
        .map_err(msg)?
        .text()
        .await
        .map_err(msg)
}

pub async fn post<U: AsRef<str>, S: Serialize, D: DeserializeOwned>(uri: U, body: S) -> Result<D> {
    let uri = uri.as_ref();
    let body =
        serde_json::to_vec(&body).map_err(|_| format!("Failed to serialise request to {uri}"))?;

    let client = reqwest::Client::new();
    let res = client
        .post(uri)
        .header("Authorization", TOKEN)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await;

    let bytes = match res {
        Ok(resp) => match resp.bytes().await {
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

pub fn api_uri<S: Display>(api_path: S) -> String {
    const API_URL: &str = "https://discord.com/api/v10";
    format!("{API_URL}{api_path}")
}
