# rust-crud-actix-mongo-api
# Rust Crud using Actix Web (v4.3) and MongoDB with JWT authentication

### Features of this demo
* Public and Private REST endpoints
* JWT token parsing from either cookie or **Authorization: Bearer** header
* Current user object injection in session
* MongoDB pool build and basic operations
* App shared Services and DB pool
* Encrypted password hashes comparison
* Read environment variables from .env

<br />

### Main Dependencies
* actix-web (4.3)
* jsonwebtoken
* mongodb
* dotenv

This boilerplate application offers the following endpoints, with JWT role-based validation demo:

| Path                     | Method |
|--------------------------|--------|
| /api/auth/login          | POST   |
| /api/auth/refresh        | POST   |
| /api/auth/validate       | GET    |
| /api/protected/admin     | GET    |
| /api/protected/user      | GET    |
| /api/public              | GET    |
| /api/user/create         | POST   |
| /api/user/get/{username} | GET    |

The **.env** file contains the mongodb connection details.

<br />


## Development environment setup

Requirements: rust toolchain, docker, docker-container

Create and start a mongodb instance with docker:

    docker-compose up -d db

<br />

## Running the Application
Run the application with the command:

    cargo run

Alternatively, you can run the command below to relaunch at any changes to the given resources:

    cargo watch -w src -w Cargo.toml -w .env -x run

<br />

## Testing

#### Create users

    curl -H 'Content-Type: application/json' -d '{"username":"admin","email":"admin@test.com","password":"abc123","roles":["Admin"]}' http://localhost:8000/api/user/create
    curl -H 'Content-Type: application/json' -d '{"username":"user","email":"user@test.com","password":"abc123","roles":["User"]}' http://localhost:8000/api/user/create

#### Login

    curl -H 'Content-Type: application/json' -d '{"email":"user@test.com","password":"abc123"}' http://localhost:8000/api/auth/login

If everything is working, and you are using Linux/MacOS/Cygwin or have access to a bash, the one-liner below can be useful to parse the token from the response:

    # login as USER
    TOKEN=$(curl -H 'Content-Type: application/json' -d '{"email":"user@test.com","password":"abc123"}' http://localhost:8000/api/auth/login | python -c 'import json,sys;print(json.load(sys.stdin)["tokens"]["access_token"])')
    echo $TOKEN
    
    # these calls below should return a 200 (OK) status code 
    curl -v -H "Authorization: Bearer ${TOKEN}" http://localhost:8000/api/public
    curl -v -H "Authorization: Bearer ${TOKEN}" http://localhost:8000/api/protected/user
    
    # this call should return a 403 (FORBIDDEN) status code
    curl -v -H "Authorization: Bearer ${TOKEN}" http://localhost:8000/api/protected/admin

    # login as ADMIN
    TOKEN=$(curl -H 'Content-Type: application/json' -d '{"email":"admin@test.com","password":"abc123"}' http://localhost:8000/api/auth/login | python -c 'import json,sys;print(json.load(sys.stdin)["tokens"]["access_token"])')

    # this call should return a 200 (OK) status code 
    curl -v -H "Authorization: Bearer ${TOKEN}" http://localhost:8000/api/protected/admin

