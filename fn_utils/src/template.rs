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
# Stage 1: Build Stage
FROM golang:1.23 as builder

# Set the working directory inside the container
WORKDIR /app

# Copy the specific function package into the container's workspace
# Replace {{FUNCTION}} with the actual function directory or file
COPY ./temp/{{FUNCTION}} .

# Initialize the Go module (if not already initialized)
RUN go mod init serverless-function

# Download dependencies early to leverage Docker cache
RUN go mod download

# Copy the application source code
COPY ./temp/{{FUNCTION}} .

# Build the Go app
RUN CGO_ENABLED=0 GOOS=linux go build -o main .

# Stage 2: Runtime Stage
FROM alpine:latest

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled binary from the builder stage
COPY --from=builder /app/main .

# Expose port 8080
EXPOSE 8080

# Set environment variables (replace with actual environment configurations)
{{ENV}}

# Command to run the application
CMD ["./main"]
"#;

pub const FUNCTION_MODULE_TEMPLATE: &str = r#"
module serverless-function

go 1.23
"#;
