
package functions

import "net/http"

// Handler for the "/{{ROUTE}}" endpoint.
func HelloApiHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write([]byte("Hello World!"))
}
