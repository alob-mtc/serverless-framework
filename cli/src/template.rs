// TEMPLATE
pub const MAIN_TEMPLATE: &str = r#"
package main

import (
	"fmt"
	"log"
	"net/http"
	"serverless-function/functions"
)

func main() {
	// Register the "/{{ROUTE}}" endpoint with the helloHandler.
	http.HandleFunc("/{{ROUTE}}", functions.{{HANDLER}})

	// Start the server on port 8080.
	fmt.Println("Server is running on port 8080...")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
"#;

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
