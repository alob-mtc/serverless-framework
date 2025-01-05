# CLI

This CLI tool provides functionality to create and deploy serverless functions.

## Installation

To install the CLI, clone the repository and build it using Cargo:

```sh
git clone <repository-url>
cd cli
cargo build --release
```

## Usage

The CLI provides the following subcommands:

Creates a new serverless function.

#### Create function

```sh
cli create-function -n <NAME> [-r <RUNTIME>]
```

Arguments

- `-n, --name <NAME>`: The name of the function to create (required).
- `-r, --runtime <RUNTIME>`: The runtime for the function (optional, default: go).

Example

```sh
cli create-function -n my-function -r go
```

#### Deploy function

Deploys an existing severless function

Usage

```sh
cli deploy-function -n <NAME>
```

Arguments

- `-n, --name <NAME>`: The name of the function to deploy (required).

Example

```sh
cli deploy-function -n my-function
```
