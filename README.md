# Actix-web DynamoDB TodoList

Simple Todo List implementation with CRUD functionality

## Prerequisites

- [Docker](https://www.docker.com/products/docker-desktop)
- [Rust & Cargo **1.34+**](https://www.rust-lang.org/tools/install)
- [NodeJS & npm](https://nodejs.org/en/download/)
- Create a `.env` file with values
  - `FRONTEND_ORIGIN=http://localhost:8080`
  - `DATABASE_NAME=tododatabase`
  - `DATABASE_URL=http://localhost:8000`
  - `TABLE_NAME=todos`
  - `BIND_ADDRESS=127.0.0.1:8080`

## Starting the Application

1. `cd client` --> `npm run build` --> `cd ..`
   - Build the ReactJS Static Files
2. `docker-compose up -d`
   - Start DynamoDB in local Docker container
3. `cargo run`
   - Build and Run the Rust Application
4. Navigate to `localhost:8080`
   - CORS is not set up for `127.0.0.1:8080`
