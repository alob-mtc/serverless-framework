// TEMPLATE
pub const MAIN_TEMPLATE: &str = r#"
package main

import (
	"fmt"
	"log"
	"net/http"
)

func main() {
	// Register the "/{{ROUTE}}" endpoint with the helloHandler.
	http.HandleFunc("/{{ROUTE}}", {{HANDLER}})

	// Start the server on port 8080.
	fmt.Println("Server is running on port 8080...")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
"#;

pub const ROUTES_TEMPLATE: &str = r#"
package main

import "net/http"

// Handler for the "/{{ROUTE}}" endpoint.
func {{HANDLER}}(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write([]byte("Hello World!"))
}
"#;

pub const DOCKERFILE_TEMPLATE: &str = r#"
# Use the official Golang image as a base image
FROM golang:1.19

# Set the working directory inside the container
WORKDIR /app

# Copy the local package files to the container's workspace
COPY ./temp/{{FUNCTION}} .

# Copy go mod and sum files
RUN go mod init serverless-function

# Download and install any required third-party dependencies
RUN go mod tidy

# Build the Go app
RUN go build -o main .

# Expose port 8080 to the outside world
EXPOSE 8080

# Env
{{ENV}}

# Command to run the executable
CMD ["./main"]
"#;

pub const FUNCTION_MODULE_TEMPLATE: &str = r#"
module serverless-function

go 1.19
"#;
