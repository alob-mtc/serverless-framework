# Serverless Framework (poc)

A modern, lightweight, and self-hosted serverless framework that lets you deploy and run functions without managing servers. **This is a proof of concept and not production-ready.**

![Serverless Architecture](./asset/serverless.png "Architecture")

## What is This Project?

This serverless framework is a proof of concept for building, deploying, and managing serverless functions in a performant and secure way. It demonstrates:

- **API Controller**: A core runtime that handles function invocation, deployment, and lifecycle management
- **Command-line Interface (CLI)**: A tool for creating, deploying, and managing serverless functions
- **Authentication System**: User management for controlled access to serverless resources

Unlike cloud provider offerings, this framework runs entirely on your infrastructure, giving you complete control over your environment and functions.

## Current Status

This project is currently in **proof of concept** stage. While it demonstrates the core concepts of a self-hosted serverless framework, it is not yet production-ready. Key limitations include:

- Limited runtime support (currently only Go)
- Basic error handling and recovery
- No production-grade monitoring or logging
- Limited scalability testing
- No production security audit

## Why Use This Framework?

- **Self-hosted**: Run everything on your own infrastructure with no vendor lock-in
- **Simple Developer Experience**: Easy-to-use CLI for function management
- **Secure by Design**: JWT-based authentication and secure function isolation
- **Lightweight**: Optimized for performance with minimal overhead
- **Function Hot-reloading**: Quick iteration on your function code
- **Docker-based Isolation**: Each function runs in its own container for security and dependency isolation

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Rust toolchain (for building the CLI)

### Running the API Controller

```sh
# Clone the repository
git clone https://github.com/yourusername/serverless.git
cd serverless

# Start the API controller and dependencies with Docker Compose
docker-compose up -d

# Or run directly with Cargo
cargo run -p api-controller
```

### Installing the CLI

```sh
# Build the CLI
cd cli
cargo build --release

# Optional: Move the binary to your PATH
cp target/release/cli /usr/local/bin/serverless-cli
```

### Creating and Deploying Your First Function

```sh
# Register a user account
serverless-cli register --email user@example.com --password your_password

# Login
serverless-cli login --email user@example.com --password your_password

# Create a new function (defaults to Go runtime)
serverless-cli create-function -n hello-world

# Deploy your function
serverless-cli deploy-function -n hello-world

# List your deployed functions
serverless-cli list
```

## Key Components

### API Controller

The API Controller is the heart of the serverless framework:

- **Function Management**: Deploys, starts, and manages function lifecycles
- **Request Routing**: Routes incoming requests to the appropriate function
- **Authentication**: Verifies user identity and permissions
- **Database**: Persists function metadata and user information
- **Redis Cache**: Tracks running function state and improves performance

### CLI Tool

The CLI offers a streamlined developer experience:

- **Function Creation**: Generate function templates and scaffolding
- **Deployment**: Package and upload functions to the API controller
- **Authentication**: Secure user management with login/registration
- **Function Listing**: View all deployed functions in a clean table format

### Docker Wrapper

Functions run in isolated Docker containers with:

- **Security Isolation**: Each function runs in its own container
- **Dependency Management**: Functions include all their dependencies
- **Runtime Support**: Currently supports Go with more runtimes planned

## Project Structure

```
serverless/
├── api-controller/       # Core runtime engine
│   ├── api/              # HTTP API implementation with Axum
│   ├── entity/           # Database entity definitions
│   ├── migration/        # Database migrations
│   ├── repository/       # Database access layer
│   └── service/          # Business logic services
├── cli/                  # Command-line interface tool
├── fn_utils/             # Shared function utilities
├── docker_wrapper/       # Docker container management
└── asset/                # Documentation assets
```

## Authentication Flow

The framework uses JWT-based authentication:

1. Users register or login through the CLI
2. The API controller validates credentials and issues a JWT token
3. The CLI stores the token locally for future requests
4. Functions are deployed and managed with authenticated requests

## Contributing

We welcome contributions to enhance this proof of concept! Here are some areas where you can make an impact:

### Areas for Contribution

- **New Runtimes**: Currently we support Go, but Python, Node.js, and other runtimes would be valuable additions
- **Function Logs**: Implementing log collection and retrieval for deployed functions
- **Metrics and Monitoring**: Adding performance measurement capabilities
- **Testing Infrastructure**: Expanding test coverage for all components
- **Documentation**: Improving guides and examples
- **Production Readiness**: Help make this proof of concept production-ready

### Development Setup

1. Clone the repository
2. Install dependencies:
   - Rust toolchain
   - Docker and Docker Compose
   - PostgreSQL and Redis (or use Docker Compose)
3. Copy `.env.example` to `.env` and configure your environment
   ```sh
   make env
   ```
4. Run database migrations:
   ```sh
   cd api-controller/migration
   cargo run
   ```
5. Start the API controller:
   ```sh
   cargo run -p api-controller
   ```

### Technical Architecture Details

- **API**: Built with Axum web framework for high performance
- **Database**: Uses SeaORM with PostgreSQL
- **Cache**: Redis for function state tracking
- **Authentication**: JWT tokens with secure validation
- **Function Isolation**: Docker containers with network controls

## Roadmap to Production Readiness

- [ ] Support for additional runtimes (Python, Node.js, etc.)
- [ ] Function logs collection and viewing
- [ ] Role-based access control
- [ ] Function versioning
- [ ] Cold start optimization
- [ ] Web dashboard for function management
- [ ] Comprehensive error handling and recovery
- [ ] Production-grade monitoring and alerting
- [ ] Security audit and hardening
- [ ] Performance optimization and load testing
- [ ] CI/CD pipeline for automated testing

## License

This project is licensed under the [LICENSE](LICENSE) file in the repository.

## Demo

[Watch Quick Demo](https://youtu.be/qLKV_cO_XhQ?si=4lmvu8frlzH1yLNX)
