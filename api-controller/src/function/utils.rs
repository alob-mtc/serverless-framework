use axum::http::{HeaderMap, Response, StatusCode as AxumStatusCode, StatusCode};
use axum::response::IntoResponse;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap as ReqwestHeaderMap;
use reqwest::StatusCode as ReqwestStatusCode;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct ScopeCall<F: FnMut()> {
    pub c: Option<F>,
}

impl<F: FnMut()> Drop for ScopeCall<F> {
    fn drop(&mut self) {
        self.c.take().unwrap()()
    }
}

pub fn defer_fn<T: FnMut()>(c: T) -> ScopeCall<T> {
    ScopeCall { c: Some(c) }
}

pub fn envs_to_string(envs: HashMap<String, String>) -> String {
    let mut envs_str = String::new();
    for (key, value) in envs {
        envs_str.push_str(&format!("ENV {}=\"{}\"\n", key, value));
    }
    envs_str
}

fn convert_status_code(reqwest_status: ReqwestStatusCode) -> AxumStatusCode {
    AxumStatusCode::from_u16(reqwest_status.as_u16())
        .unwrap_or(AxumStatusCode::INTERNAL_SERVER_ERROR)
}

fn convert_axum_headers_to_req_header(headers: HeaderMap) -> ReqwestHeaderMap {
    let mut header_res = ReqwestHeaderMap::new();
    for (hn, hv) in headers.iter() {
        header_res.append(hn, hv.clone());
    }
    header_res
}

fn convert_req_header_to_axum_headers(req_headers: &ReqwestHeaderMap, res_headers: &mut HeaderMap) {
    for (hn, hv) in req_headers.iter() {
        res_headers.append(hn, hv.clone());
    }
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
    headers: HeaderMap,
    body: serde_json::Value,
) -> impl IntoResponse {
    let client = Client::new();
    let response = client
        .post(&create_url(addr, key, query))
        .headers(convert_axum_headers_to_req_header(headers))
        .json(&body)
        .send();

    match response {
        Ok(res) => {
            let status = convert_status_code(res.status());
            let req_headers = res.headers().clone();
            match res.text() {
                Ok(text) => {
                    let mut response = Response::builder().status(status).body(text).unwrap();

                    let headers = response.headers_mut();
                    convert_req_header_to_axum_headers(&req_headers, headers);
                    response
                }
                Err(_) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Failed to read response".to_string())
                    .unwrap(),
            }
        }
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to make request".to_string())
            .unwrap(),
    }
}

pub fn create_fn_files_base(name: &str, _runtime: &str) -> std::io::Result<(PathBuf, File)> {
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

    Ok((path, main_file))
}
