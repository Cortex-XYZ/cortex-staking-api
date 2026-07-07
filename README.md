# Cortex Staking API

A production-focused, multi-chain staking platform developed by Cortex Global.

The Cortex Staking API is designed to provide a secure, scalable foundation for validator operations, staking workflows, accounting, infrastructure management, and developer tooling across multiple blockchain ecosystems.

Rather than building isolated APIs for each blockchain, Cortex follows a platform-first architecture. Core capabilities—including authentication, authorization, administration, auditing, rate limiting, pagination, filtering, and request tracing—are implemented once and shared across every supported network.

The first blockchain integration is **Monad**, but the long-term architecture is intentionally chain-agnostic to support Ethereum, Solana, Sui, Bitcoin, and additional proof-of-stake networks.

---

# Documentation

Project documentation is organized as follows:

* `README.md` — Project overview, architecture, and development roadmap
* `CONTRIBUTING.md` — Development workflow and contribution guidelines
* `migrations/schema.md` — Database schema documentation
* `skills/` — Architecture, coding standards, and design documentation

All contributors should review `CONTRIBUTING.md` before submitting changes.

---

# Current Status

## ✅ Implemented

### Core Platform

* Rust workspace architecture
* Modular crate structure
* Actix Web API server
* Shared application state (`AppState`)
* SQLx PostgreSQL integration
* Dockerized local PostgreSQL development
* Repository → Service → Route architecture
* Request ID middleware
* Standardized API error responses
* In-memory request rate limiting
* OpenAPI / Swagger integration
* Health and readiness endpoints

### Authentication & Authorization

* Database-backed API key authentication
* Organization-based authorization
* Scope-based authorization
* Cortex administrator authorization
* Partner authorization
* User account model
* API key hashing (SHA-256)
* One-time plaintext API key generation
* API key rotation
* API key revocation
* Soft deletion of API keys

### Administrative Platform

#### Organizations

* Create
* List
* Retrieve
* Update
* Soft delete

#### API Keys

* Create
* List
* Retrieve
* Rotate
* Revoke
* Soft delete

#### Users

* List
* Retrieve
* Update
* Soft delete

### Audit & Compliance

* Audit log database
* Audit repository
* Audit service layer
* Automatic audit logging for administrative mutations
* Request ID tracking
* Actor organization tracking
* Actor API key tracking
* Structured JSON change history

### API Platform Features

* Pagination
* Sorting
* Filtering foundation
* Standard response envelopes
* Request correlation IDs
* Service-layer business logic
* Repository abstraction

### Database

Current database entities include:

* Organizations
* Users
* API Keys
* API Key Scopes
* Audit Logs

---

## 🚧 In Progress

### Platform

* Advanced filtering across administrative resources
* Audit log query endpoints
* Redis-backed distributed rate limiting
* Response logging middleware
* Expanded integration test coverage

### Monad

* Service layer implementation
* Validator endpoints
* Staking workflows
* Reward reporting
* Accounting endpoints

---

## 📋 Planned

### Developer Platform

* User self-service API key management
* Organization invitations
* Role and permission management
* API usage analytics
* Developer dashboard
* Public developer portal

### Multi-Chain Support

* Ethereum
* Solana
* Sui
* Bitcoin
* Additional proof-of-stake networks

### Platform Services

* Validator accounting engine
* Background workers
* Distributed caching
* Prometheus metrics
* Grafana dashboards
* Production deployment pipeline
* CI/CD automation
* Disaster recovery procedures

### Future Products

* Partner dashboard
* Validator operations dashboard
* Billing and subscription management
* Usage-based quotas
* Self-service onboarding
* Public API marketplace

# Architecture

The Cortex Staking API follows a layered architecture that separates HTTP concerns from business logic and database access.

Each request flows through middleware, authentication, authorization, services, and repositories before interacting with PostgreSQL.

```text
HTTP Request
        │
        ▼
Request ID Middleware
        │
        ▼
Authentication
        │
        ▼
Rate Limiter
        │
        ▼
Authorization
        │
        ▼
Handler
        │
        ▼
Service
        │
        ▼
Repository
        │
        ▼
PostgreSQL
```

Each layer has a single responsibility.

| Layer          | Responsibility                                                |
| -------------- | ------------------------------------------------------------- |
| Middleware     | Cross-cutting concerns such as request IDs and rate limiting  |
| Authentication | Validate API keys and build the authenticated request context |
| Authorization  | Verify organization type and scopes                           |
| Handlers       | Validate HTTP requests and return HTTP responses              |
| Services       | Business logic and orchestration                              |
| Repositories   | Database access using SQLx                                    |
| Database       | Persistent storage                                            |

This architecture keeps HTTP concerns separate from business logic and allows future blockchain integrations to reuse the same platform components.

---

# Project Structure

```text
cortex-staking-api/
│
├── crates/
│   │
│   ├── api/
│   │   ├── docs/
│   │   ├── error/
│   │   ├── extractors/
│   │   ├── middleware/
│   │   ├── pagination/
│   │   ├── routes/
│   │   ├── app.rs
│   │   ├── config.rs
│   │   └── state.rs
│   │
│   ├── auth/
│   │   ├── api_key.rs
│   │   ├── extractor.rs
│   │   ├── middleware.rs
│   │   ├── model.rs
│   │   └── scopes.rs
│   │
│   ├── db/
│   │   ├── api_key_repository.rs
│   │   ├── audit_repository.rs
│   │   ├── auth_repository.rs
│   │   ├── organization_repository.rs
│   │   ├── pagination.rs
│   │   └── user_repository.rs
│   │
│   ├── services/
│   │   ├── api_key_service.rs
│   │   ├── audit_service.rs
│   │   ├── organization_service.rs
│   │   └── user_service.rs
│   │
│   └── chains/
│       └── monad/
│
├── migrations/
├── skills/
├── Cargo.toml
└── README.md
```

The goal is to isolate business logic from transport concerns while making each blockchain implementation independent of every other chain.

---

# Request Lifecycle

Every request follows the same processing pipeline.

```text
Incoming Request
        │
        ▼
Generate Request ID
        │
        ▼
Authenticate API Key
        │
        ▼
Enforce Rate Limit
        │
        ▼
Authorize Organization & Scopes
        │
        ▼
Execute Handler
        │
        ▼
Service Layer
        │
        ▼
Repository Layer
        │
        ▼
Database
        │
        ▼
Response
```

Administrative mutations additionally write an audit log before the response is returned.

This consistent request lifecycle means every future endpoint automatically benefits from authentication, authorization, request tracing, audit logging, and rate limiting.

---

# Route Structure

Routes are grouped by responsibility rather than by implementation.

## Health

```text
GET /healthz
GET /readyz
```

Used for:

* Kubernetes readiness probes
* Load balancer health checks
* Infrastructure monitoring

---

## Administration

Administrative endpoints require a Cortex administrator API key.

### Organizations

```text
GET    /admin/organizations
GET    /admin/organizations/{id}

POST   /admin/organizations

PATCH  /admin/organizations/{id}

DELETE /admin/organizations/{id}
```

Supports:

* Organization onboarding
* Organization management
* Soft deletion

---

### API Keys

```text
GET    /admin/api-keys
GET    /admin/api-keys/{id}

POST   /admin/api-keys

POST   /admin/api-keys/{id}/rotate

POST   /admin/api-keys/{id}/revoke

DELETE /admin/api-keys/{id}
```

Features include:

* One-time plaintext key generation
* Secure key rotation
* Key revocation
* Soft deletion
* Pagination
* Sorting
* Filtering

---

### Users

```text
GET    /admin/users
GET    /admin/users/{id}

PATCH  /admin/users/{id}

DELETE /admin/users/{id}
```

These endpoints form the foundation of the future developer portal.

---

### Audit

Administrative mutations are automatically recorded in the audit log.

Planned endpoints:

```text
GET /admin/audit-logs
GET /admin/audit-logs/{id}
```

These endpoints will support pagination, filtering, and compliance reporting.

---

## Monad

The first blockchain implementation lives under `/monad`.

Planned endpoints include:

```text
GET  /monad/validators
GET  /monad/validators/{id}

POST /monad/stake
POST /monad/unstake

GET  /monad/rewards
GET  /monad/accounting
```

Future chains (Ethereum, Solana, Sui, Bitcoin, etc.) will expose similar route groups while reusing the shared authentication, authorization, auditing, pagination, filtering, and rate-limiting infrastructure.


# Authentication

Every request to the Cortex Staking API must be authenticated using an API key.

API keys belong to either an organization or an individual user and are validated against the database on every request.

```http
Authorization: Bearer <api_key>
```

Authentication is performed before any route handler executes.

---

## Organization Types

Organizations define the trust level of an API key.

Current organization types are:

```text
Cortex
Partner
User
```

Typical permissions include:

| Organization | Intended Use                                            |
| ------------ | ------------------------------------------------------- |
| Cortex       | Administrative platform management                      |
| Partner      | External staking partners and validator operators       |
| User         | Individual developer accounts (future developer portal) |

---

## Scope-Based Authorization

Every API key contains one or more scopes that determine which operations are permitted.

Example scopes include:

```text
admin:*

admin:organizations:read
admin:organizations:write

admin:api-keys:read
admin:api-keys:write

admin:users:read
admin:users:write

monad:validators:read
monad:validators:write

monad:staking:write
monad:accounting:read
```

Scopes are evaluated after authentication but before the request reaches the handler.

This allows the same organization to have multiple API keys with different privilege levels.

---

# Security Model

Security is a core design principle of the Cortex platform.

Current protections include:

* API keys stored as SHA-256 hashes
* Plaintext API keys shown exactly once during creation
* Organization-based authorization
* Scope-based authorization
* Soft deletion of organizations, users, and API keys
* Administrative audit logging
* Request correlation IDs
* Standardized error responses
* Request rate limiting
* External HTTP clients configured with connection and request timeouts
* No plaintext secrets written to application logs
* No blockchain private keys stored by the API

The platform is designed to follow the principle of least privilege by default.

---

# Audit Logging

Administrative mutations automatically generate audit log entries.

Each audit record captures:

* Timestamp
* Request ID
* Acting API key
* Acting organization
* Action performed
* Resource type
* Resource identifier
* Previous values (when applicable)
* Updated values (when applicable)

Examples of audited operations include:

```text
organization.created
organization.updated
organization.deleted

api_key.created
api_key.rotated
api_key.revoked
api_key.deleted

user.updated
user.deleted
```

Plaintext API keys are never written to the audit log.

---

# Rate Limiting

Every API key stores an associated request limit.

```text
rate_limit_per_minute
```

Each authenticated request is evaluated against the current one-minute window before reaching the handler.

If the limit is exceeded, the API responds with:

```http
HTTP 429 Too Many Requests
```

The current implementation uses an in-memory rate limiter suitable for development and single-instance deployments.

Future production deployments will replace this with a Redis-backed distributed rate limiter to support multiple API servers.

---

# Pagination

Collection endpoints support consistent pagination.

Example:

```text
GET /admin/organizations?page=1&page_size=25
```

Supported parameters:

| Parameter   | Description                        |
| ----------- | ---------------------------------- |
| `page`      | Page number (1-based)              |
| `page_size` | Maximum number of records returned |
| `sort`      | Sort column                        |
| `direction` | `asc` or `desc`                    |

Responses include pagination metadata.

Example:

```json
{
  "data": [
    ...
  ],
  "pagination": {
    "page": 1,
    "page_size": 25,
    "total_items": 143,
    "total_pages": 6
  }
}
```

---

# Filtering

Administrative collection endpoints support server-side filtering.

### Organizations

Supported filters:

```text
status
kind
name
```

Example:

```text
GET /admin/organizations?status=active&kind=partner
```

---

### API Keys

Supported filters:

```text
organization_id
status
scope
created_after
last_used_after
```

Example:

```text
GET /admin/api-keys?status=active&scope=monad:staking:write
```

Filtering is performed within the database to minimize network overhead and improve performance.

---

# Standard Error Responses

All endpoints return a consistent error format.

Example:

```json
{
  "error": {
    "code": "organization_not_found",
    "message": "Organization not found",
    "request_id": "8cbccf7d-99cb-4376-b6b8-c5df7f60a0f5"
  }
}
```

Including the request ID in every error response allows operators to quickly correlate client-side failures with server logs and audit records.

---

# OpenAPI Documentation

The Cortex Staking API follows an OpenAPI-first development model.

Documentation is generated automatically using:

* utoipa
* utoipa-swagger-ui

Swagger UI is available locally at:

```text
http://127.0.0.1:8080/documentation/
```

Available specifications:

```text
Health
Admin
Monad
```

Raw OpenAPI documents:

```text
/api-docs/health/openapi.json
/api-docs/admin/openapi.json
/api-docs/monad/openapi.json
```

Every endpoint should be documented as it is implemented so that the OpenAPI specification remains the authoritative reference for API consumers.

# Local Development

## Running the API

Start the development server:

```bash
cargo run -p cortex-staking-api
```

The API will be available at:

```text
http://127.0.0.1:8080
```

---

## Health Checks

Verify the application is running:

```bash
curl http://127.0.0.1:8080/healthz
```

Verify dependencies are available:

```bash
curl http://127.0.0.1:8080/readyz
```

---

## Swagger UI

Interactive API documentation is available at:

```text
http://127.0.0.1:8080/documentation/
```

Swagger specifications are automatically generated during development.

---

## Database

The project uses PostgreSQL with SQLx.

Typical development workflow:

1. Start the local PostgreSQL instance.
2. Apply database migrations.
3. Seed development data.
4. Run the API.
5. Begin development.

Development seed data includes Cortex organizations, partner organizations, users, and API keys for local testing.

---

# Design Principles

The Cortex platform follows several architectural principles intended to keep the codebase maintainable as additional blockchain networks are added.

---

## Keep `main.rs` Small

`main.rs` should only be responsible for:

* Loading configuration
* Initializing logging
* Building shared application state
* Starting the HTTP server

No business logic, routing, SQL, or blockchain code should exist in `main.rs`.

---

## Thin Handlers

Handlers are responsible only for HTTP concerns.

Handlers should:

* Validate requests
* Extract authentication context
* Call services
* Return HTTP responses

Handlers should **never** contain business logic.

---

## Service-Oriented Business Logic

Business rules belong in the service layer.

Examples include:

* Creating organizations
* Generating API keys
* Rotating credentials
* Writing audit logs
* Staking workflows
* Reward calculations
* Validator accounting

Services orchestrate repositories but remain independent of HTTP.

---

## Repository Pattern

Repositories own all database access.

Repositories should:

* Execute SQL
* Map rows into Rust models
* Return typed results

Repositories should **not** implement business rules.

---

## Middleware First

Cross-cutting concerns belong in middleware rather than handlers.

Current middleware responsibilities include:

* Request IDs
* Authentication
* Rate limiting

Future middleware will include:

* Request logging
* Response logging
* Distributed rate limiting
* Metrics collection

---

## Chain Isolation

Every blockchain implementation should remain isolated from the core platform.

Current structure:

```text
crates/
└── chains/
    └── monad/
```

Future chains will follow the same pattern:

```text
crates/
└── chains/
    ├── ethereum/
    ├── solana/
    ├── sui/
    ├── bitcoin/
    └── ...
```

Core authentication, authorization, auditing, pagination, filtering, request tracing, and rate limiting should never require chain-specific modifications.

---

## Shared Platform Components

Every blockchain should automatically inherit the platform capabilities provided by the core system.

These include:

* Authentication
* Authorization
* Request IDs
* Audit logging
* Pagination
* Filtering
* Standardized errors
* Rate limiting
* OpenAPI documentation

New blockchain integrations should focus exclusively on blockchain-specific functionality.

---

# Development Roadmap

The project follows a platform-first development strategy.

## ✅ Phase 1 — Platform Foundation

Completed:

* Workspace architecture
* Authentication
* Authorization
* Organization management
* API key management
* User management
* Audit logging
* Request IDs
* Rate limiting
* Standardized errors
* Pagination
* Filtering
* Service layer architecture

---

## 🚧 Phase 2 — Platform Maturity

Current priorities:

* Audit log query endpoints
* Advanced filtering
* Redis-backed distributed rate limiting
* Response logging middleware
* Expanded integration testing

---

## 📋 Phase 3 — Monad

Implement the first production blockchain integration.

Planned functionality:

* Validator discovery
* Validator health
* Staking
* Unstaking
* Reward reporting
* Validator accounting
* RPC integration

---

## 📋 Phase 4 — Multi-Chain Platform

Expand the platform beyond Monad.

Target networks include:

* Ethereum
* Solana
* Sui
* Bitcoin
* Additional proof-of-stake ecosystems

Each chain should reuse the shared Cortex platform while implementing only its own blockchain-specific services.

---

## 📋 Phase 5 — Developer Platform

Build a complete self-service ecosystem.

Planned capabilities:

* Organization onboarding
* User self-service
* API key management
* Developer dashboard
* Usage analytics
* Billing and subscriptions
* Public developer portal

---

# Contributing

Contributors are encouraged to follow the existing architectural patterns and keep the platform modular.

When adding new functionality:

1. Create or extend database migrations if needed.
2. Implement repository methods.
3. Implement business logic in the service layer.
4. Add HTTP handlers.
5. Document routes with OpenAPI annotations.
6. Add audit logging for administrative mutations.
7. Add integration tests.
8. Update documentation.

Following these conventions ensures the platform remains consistent as it grows.

---

# Project Vision

The Cortex Staking API is intended to become a production-grade backend platform for validator operations across multiple blockchain ecosystems.

Rather than building separate APIs for each network, Cortex provides a shared platform that standardizes authentication, authorization, auditing, administration, observability, and developer experience.

As additional blockchain integrations are added, they inherit these platform capabilities automatically, allowing development to focus on blockchain-specific logic instead of rebuilding common infrastructure.

The long-term vision is to provide a secure, scalable, and extensible foundation for staking, validator management, accounting, and infrastructure services across the broader blockchain ecosystem.
