use axum::extract::Query;
use axum::http::{StatusCode as AxumStatusCode, StatusCode};
use reqwest::blocking::Client;
use reqwest::StatusCode as ReqwestStatusCode;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::Path;

fn convert_status_code(reqwest_status: ReqwestStatusCode) -> AxumStatusCode {
    AxumStatusCode::from_u16(reqwest_status.as_u16())
        .unwrap_or(AxumStatusCode::INTERNAL_SERVER_ERROR)
}

// random 4 digit port generator
pub fn random_port() -> String {
    let port = rand::random::<u16>() % 1000 + 8000;
    port.to_string()
}

fn create_url(addr: &str, key: &str, query: HashMap<String, String>) -> String {
    let mut url = format!("http://{addr}/{key}");
    for (k, v) in query.iter() {
        url.push_str(&format!("?{k}={v}"));
    }
    url
}

pub fn make_request(
    addr: &str,
    key: &str,
    query: HashMap<String, String>,
    body: serde_json::Value,
) -> (StatusCode, String) {
    let client = Client::new();
    let response = client
        .post(&create_url(addr, key, query))
        .json(&body)
        .send();

    match response {
        Ok(res) => {
            let status = convert_status_code(res.status());
            match res.text() {
                Ok(text) => (status, text),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read response".to_string(),
                ),
            }
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to make request".to_string(),
        ),
    }
}

pub fn create_fn_files(name: &str, _runtime: &str) -> std::io::Result<Vec<File>> {
    let path = Path::new("temp");
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Folder '{}' already exists.", name),
        ));
    }

    fs::create_dir(&path)?;

    let path = path.join(name);
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Folder '{}' already exists.", name),
        ));
    }

    fs::create_dir(&path)?;

    let main_file_path = path.join("main.go");
    let main_file = File::create(&main_file_path)?;

    let routes_path = path.join("functions");
    fs::create_dir(&routes_path)?;

    let routes_file_path = routes_path.join("routes.go");
    let routes_file = File::create(&routes_file_path)?;

    Ok(vec![main_file, routes_file])
}
