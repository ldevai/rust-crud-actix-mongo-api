# rust-crud-actix-mongo-api

A JWT-secured REST API in Rust with actix-web 4 and MongoDB: bcrypt passwords,
access + refresh tokens read from an HttpOnly cookie or an `Authorization: Bearer`
header, a `CurrentUser` extractor, and role checks (`User` / `Admin`) on protected routes.

> Maintained in [devai-io/devai_boilerplates](https://github.com/devai-io/devai_boilerplates/tree/main/rust-crud-actix-mongo-api),
> the public home of every [devai.io](https://devai.io) project; this repo carries the same code.

## Run

    git clone https://github.com/ldevai/rust-crud-actix-mongo-api.git
    cd rust-crud-actix-mongo-api
    docker compose up --build

The API answers on http://localhost:8080 (`curl localhost:8080/health` → `ok`).
MongoDB keeps its state in `./data/mongo`; the unique indexes on `users.email` and
`users.username` are ensured on every start.

Without Docker: run a MongoDB, export the variables from `.env.example`, then
`cargo run` (Rust 1.98, the toolchain the Dockerfile pins).

## How it works

| Method | Path                    | Auth       | Result                                                           |
|--------|-------------------------|------------|------------------------------------------------------------------|
| GET    | `/health`               | —          | `200 ok`                                                         |
| POST   | `/api/user/create`      | —          | `{email, username, password}` → `201` user, `409` if taken       |
| POST   | `/api/auth/login`       | —          | `{email, password}` → `200 {email, username, roles, tokens}` + `token` cookie, `401` if wrong |
| POST   | `/api/auth/refresh`     | —          | `{refresh_token}` → `200` new token pair, `401` if used/revoked  |
| GET    | `/api/auth/validate`    | any role   | `200` current user, `401` without a valid token                  |
| GET    | `/api/user/{username}`  | any role   | `200` user — yourself, or anyone if you are `Admin` (`403` otherwise) |
| GET    | `/api/public`           | optional   | `200 {username, endpoint_security}`, `username` is `null` when anonymous |
| GET    | `/api/protected/user`   | any role   | `200`, or `401`                                                  |
| GET    | `/api/protected/admin`  | `Admin`    | `200`, `401` without a token, `403` for a plain `User`           |

- **Tokens** — login returns an access token (15 min) and a refresh token (7 days),
  both HS256 JWTs signed with `AUTH_SECRET`. Verification pins HS256 and requires
  `exp`, so `alg: none` or re-signed tokens are rejected.
- **Sessions** — each token carries the user's current `session_id`. Logging in again
  or refreshing replaces it, which revokes every older token; a refresh token works once.
- **`CurrentUser`** (`src/security.rs`) — an actix extractor: reads the Bearer header
  or the `token` cookie, verifies the JWT, and loads the user from MongoDB. Put it in
  a handler's arguments to require login, `Option<CurrentUser>` to make it optional,
  and call `current_user.require(Role::Admin)?` for a role check.
- **Passwords** — bcrypt (cost 12), hashed on actix's blocking pool so slow hashing
  never stalls request handling.
- **Roles** — sign-up always creates a `User`. Promote an admin on the database;
  roles are read from MongoDB on every request, so it applies immediately:

      docker compose exec db mongosh demo --eval \
        'db.users.updateOne({username: "admin"}, {$set: {roles: ["User", "Admin"]}})'

- **Errors** are always JSON: `{"error": "message"}`.

Try it:

    curl -X POST localhost:8080/api/user/create -H 'content-type: application/json' \
      -d '{"email":"user@test.com","username":"user","password":"supersecret"}'
    TOKEN=$(curl -s localhost:8080/api/auth/login -H 'content-type: application/json' \
      -d '{"email":"user@test.com","password":"supersecret"}' | jq -r .tokens.access_token)
    curl -H "Authorization: Bearer $TOKEN" localhost:8080/api/protected/user    # 200
    curl -H "Authorization: Bearer $TOKEN" localhost:8080/api/protected/admin   # 403

## Layout

    src/main.rs             env, MongoDB connection, routes
    src/security.rs         JWT keys, bcrypt, the CurrentUser extractor
    src/auth/service.rs     login, refresh and token → user, with session rotation
    src/users/service.rs    sign-up, lookup, unique indexes
    src/test_controller.rs  the public / user / admin demo endpoints
    src/errors.rs           ApiError → {"error": ...} responses

## Deploy

Fork this repo (or push a copy to your own GitHub repo) and the shipped workflow
(`.github/workflows/ci.yml`) tests the compose stack, publishes the image to
GHCR, and — once you set the `DEPLOY_HOST` / `DEPLOY_USER` variables and
`DEPLOY_KEY` secret — deploys it to your server over ssh. Set a long random
`AUTH_SECRET` on the server; the one in `compose.yaml` is for local use only.

---
Part of [devai.io](https://devai.io) — Rust API boilerplates, alongside
[`rust-crud-sql-api`](https://github.com/devai-io/devai_boilerplates/tree/main/rust-crud-sql-api) and
[`rust-crud-nosql-api`](https://github.com/devai-io/devai_boilerplates/tree/main/rust-crud-nosql-api).
