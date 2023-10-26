# Backend showcase with JsonPlaceholder

This is a backend project built with Rust, Reqwest, Tokio and Actix-web and meant as a showcase.

The goal is to provide a secure REST api for reading and updating User info in [JsonPlaceholder](https://jsonplaceholder.typicode.com).

> Because JsonPlaceholder does not support editing or saving user info, we use MongoDb under the hood. This part of the service is meant to be invisible to the end user, hence all APIs act as if they only communicate with JsonPlaceholder.

#### Swagger

> Not yet implemented.

Swagger is provided at path `/swagger`.

#### Documentation

This README-file is provided via [Docsify](https://docsify.js.org/#/quickstart) at the project root url `/`.

## Authentication

> Not yet implemented.

The REST apis provided by this project have been secured with OIDC and require a bearer-token provided by an identity manager.

This project has been tested and built against [Keycloak](https://www.keycloak.org).

## Operations

Documentation for all the data flows in this project.

For info about the REST apis, see [swagger](#swagger)

### Get users

Get a list of all users.

> Roles allowed: "admin", "user"

```mermaid
sequenceDiagram
    actor U as User
    participant B as Backend
    participant M as MongoDb
    participant J as JsonPlaceholder
    
    U ->> B: GET-request with bearer-token.
    B ->> M: Request for all users.
    M -->> B: All saved users as a list.
    B ->> J: Request for all users.
    J -->> B: Users as a list or error code.
    B ->> B: Combine the two user lists and make sure there are no duplicates.
    B -->> U: Users as a list or error code.
```

### Get user with id

Get user with specific id.

> Roles allowed: "admin", "user"

```mermaid
sequenceDiagram
    actor U as User
    participant B as Backend
    participant M as MongoDb
    participant J as JsonPlaceholder
    
    U ->> B: GET-request with bearer-token and user id.
    B ->> M: Search user with id.
    alt User is found
        M -->> B: User info.
    else User is not found.
        M -->> B: Empty result.
        B ->> J: Request for user with id.
        J -->> B: User or error code.
    end
    B -->> U: User or error code.
```

### Create new user

Create new user.

> Roles allowed: "admin"

```mermaid
sequenceDiagram
    actor U as User
    participant B as Backend
    participant M as MongoDb
    participant J as JsonPlaceholder
    
    U ->> B: POST-request with bearer-token and new user info.
    B ->> B: Generate user if for new user.
    B ->> M: Save new user info.
    B -->> U: Saved user with status code 201.
```

### Update existing user

Update existing user.

> Roles allowed: "admin"

```mermaid
sequenceDiagram
    actor U as User
    participant B as Backend
    participant M as MongoDb
    participant J as JsonPlaceholder

    U ->> B: PATCH-request with bearer-token and updated user info.
    B ->> M: Check if user with id exists.
    alt User found in MongoDb 
        M -->> B: User info.
        B ->> B: Update user model.
        B ->> M: Save changes.
        B -->> U: 200-OK
    else User not found in MongoDb
        M -->> B: Empty result.
        B ->> J: Search for user with given id.
        alt User found
            J -->> B: User info.
            B ->> B: Update user model.
            B ->> M: Save changes.
            B -->> U: 200-OK
        else User not found
            J -->> B: Error.
            B -->> U: Error.
        end
    end
```
