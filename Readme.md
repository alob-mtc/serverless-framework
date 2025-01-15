# serverless framework (POC)

This is a simple serverless framework(runtime and cli) that can be used to run serverless functions in a durable way.


![Serverless Architecture](./asset/serverless.png "Architecture")

## Running

### API Controller
```sh
$ cargo run -p api-controller
```

Supported languages:
- [x] Golang

TODO:
- [ ] Create docker wrapper
    - [x] Add docker wrapper
    - [ ] Collect container logs and store them 
- [x] Create API gateway
    - [ ] New Process controller 
    - [x] Receive incoming request (function invocation)
        - [x] Signer start of function process
        - [x] Forward request to function process
        - [x] Retrieve response from function process and bobble it backup
    - [x] Function call
        - [x] Forward function response header
        - [x] Forward request header to function
        - [x] Forward query param and body to function
        - [x] Add running function store
        - [x] Create a process that keeps the stare up to date
        - [x] Create a store that track running functions
        - [x] Dynamically create function port
        - [x] Start function If not started
        - [x] Check Is function already started
- [x] Create CLI
    - [x] Create function
    - [x] Support environment variables
    - [x] Deploy function
    - [x] Make sure the function uses the same go version as the docker image
    - [x] Support multiple files
    - [x] Support go module system
- [x] Create a new architecture for the new direction of the project
- [ ] Create a road map for the project