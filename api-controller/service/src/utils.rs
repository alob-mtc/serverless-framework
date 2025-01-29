use axum::body::Body;
use axum::http::{
    HeaderMap, Request as AxumRequest, Response as AxumResponse, StatusCode as AxumStatusCode,
    StatusCode,
};
use axum::response::IntoResponse;
use hyper::body::to_bytes;
use reqwest::header::HeaderMap as ReqwestHeaderMap;
use reqwest::Client;
use reqwest::StatusCode as ReqwestStatusCode;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use urlencoding::encode;

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

fn convert_req_header_to_axum_headers(
    req_headers: &mut ReqwestHeaderMap,
    res_headers: &mut HeaderMap,
) {
    req_headers.remove(http::header::TRANSFER_ENCODING);

    for (hn, hv) in req_headers.iter() {
        println!("hn: {} hv: {:?}", hn.to_string(), hv.to_str());
        res_headers.append(hn, hv.clone());
    }
}

// random 4 digit port generator
pub fn random_port() -> String {
    let port = rand::random::<u16>() % 1000 + 8000;
    port.to_string()
}

fn create_url(addr: &str, key: &str, query: HashMap<String, String>) -> String {
    let mut url = format!("http://{}/{}", addr, key);

    if !query.is_empty() {
        let query_string = query
            .iter()
            .map(|(k, v)| format!("{}={}", encode(k), encode(v))) // Escape query parameters
            .collect::<Vec<_>>()
            .join("&");

        url.push('?');
        url.push_str(&query_string);
    }

    url
}

pub async fn make_request(
    addr: &str,
    key: &str,
    query: HashMap<String, String>,
    headers: HeaderMap,
    req: AxumRequest<Body>,
) -> impl IntoResponse {
    let client = Client::new();
    // Determine which HTTP method the incoming request has
    let response_result = match req.method() {
        &http::Method::GET => {
            client
                .get(create_url(addr, key, query))
                .headers(convert_axum_headers_to_req_header(headers))
                .send()
                .await
        }
        &http::Method::POST => {
            let body_bytes = match to_bytes(req.into_body()).await {
                Ok(bytes) => bytes,
                Err(_) => {
                    return AxumResponse::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body("Could not read request body".to_owned())
                        .unwrap();
                }
            };

            client
                .post(create_url(addr, key, query))
                .headers(convert_axum_headers_to_req_header(headers))
                .body(body_bytes)
                .send()
                .await
        }
        _ => {
            return AxumResponse::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(format!(
                    "We don't currently support {} functions",
                    req.method().to_string()
                ))
                .unwrap();
        }
    };

    // Handle the result of the downstream request
    let response = match response_result {
        Ok(res) => {
            let status = convert_status_code(res.status());
            let mut req_headers = res.headers().clone();

            // Read downstream response text
            match res.text().await {
                Ok(text) => {
                    let mut response = AxumResponse::builder().status(status).body(text).unwrap();

                    // Convert reqwest response headers back into Axum response headers
                    let headers = response.headers_mut();
                    convert_req_header_to_axum_headers(&mut req_headers, headers);

                    response
                }
                Err(_) => AxumResponse::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Failed to read downstream response".to_owned())
                    .unwrap(),
            }
        }
        Err(_) => AxumResponse::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to make downstream request".to_string())
            .unwrap(),
    };

    response
}

pub fn create_fn_files_base(path: &PathBuf, name: &str, _runtime: &str) -> std::io::Result<File> {
    if path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Folder '{}' already exists.", name),
        ));
    }

    fs::create_dir(&path)?;

    let main_file_path = path.join("main.go");
    let main_file = File::create(&main_file_path)?;

    Ok(main_file)
}
