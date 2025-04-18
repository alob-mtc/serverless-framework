/*!
This module serves as the base for host management for the CLI.
- Handles injecting the correct host at build time.
*/

/// HOST_BASE is the base URL for the API server
const HOST_BASE: &str = "http://127.0.0.1:3000";

/// Generates the URL for the login endpoint
pub fn auth_login_url() -> String {
    format!("{}/auth/login", HOST_BASE)
}
/// Generates the URL for the register endpoint
pub fn auth_register_url() -> String {
    format!("{}/auth/register", HOST_BASE)
}
/// Generates the URL for the function upload endpoint
pub fn function_upload_url() -> String {
    format!("{}/functions/upload", HOST_BASE)
}
/// Generates the URL for the function list endpoint
pub fn function_list_url() -> String {
    format!("{}/functions", HOST_BASE)
}
