// TEMPLATE
pub const ROUTES_TEMPLATE: &str = r#"
package functions

import "net/http"

// Handler for the "/{{ROUTE}}" endpoint.
func {{HANDLER}}(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write([]byte("Hello World!"))
}
"#;
