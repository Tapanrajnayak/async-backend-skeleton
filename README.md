# async-backend-skeleton

This project is a small but production-minded async backend service written in Rust.
It reflects patterns I've used in real control-plane and distributed systems, adapted to Rust's ownership and concurrency model, with an emphasis on correctness, idempotency, and secure state handling.

I built it as a financial transaction ledger - not because it needs to move money, but because ledger semantics force you to get the hard things right: idempotent writes, state machine invariants, and concurrent access without data races. If you can model a ledger cleanly, you can model most backend domains.

I optimized for **clarity of intent** over cleverness. Every module has a single responsibility, error types map directly to HTTP semantics, and the domain logic is testable without touching any I/O layer.

What I intentionally didn't build: there's no database, no auth middleware, no container setup, no CI pipeline. Those are configuration, not design. I wanted the architecture to speak for itself without the noise.

## Design Notes

**Why Axum.** Axum builds on `tower::Service`, which means middleware, extractors, and handlers compose as types rather than as macros or runtime reflection. This makes the compiler your first line of defense - a handler that doesn't satisfy its trait bounds won't compile. That matters when you're maintaining services at scale, not just writing them once.

**Why explicit state machines.** Transaction status transitions (`Pending → Completed | Failed | Cancelled`, nothing backward) are enforced in the domain layer, not at the API boundary. This means the invariant holds regardless of how the service is called - HTTP, tests, a future gRPC layer, or internal batch jobs. Encoding allowed transitions in a `can_transition_to` method makes illegal states unrepresentable in practice, even without reaching for session types.

**How Rust helps prevent entire bug classes.** The storage layer uses `Arc<RwLock<HashMap>>` - concurrent readers, exclusive writers, zero `unsafe`. In Go or Java this pattern is easy to get wrong silently (forgotten mutex, map access outside a lock). In Rust, the borrow checker won't let you touch the inner data without going through the lock. The `Storage` trait uses `Send + Sync + 'static` bounds, so the compiler proves thread safety at build time rather than hoping your tests catch a race at runtime.

**What I'd improve with more time.**
- Swap in-memory storage for PostgreSQL behind the same `Storage` trait - the interface is already designed for it.
- Add request-scoped tracing with correlation IDs propagated through `tower-http` middleware.
- Introduce pagination and cursor-based listing instead of returning all records.
- Add property-based tests (via `proptest`) for state machine transitions to cover edge cases exhaustively.
- Wire in OpenTelemetry spans for distributed tracing across service boundaries.

## API

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `POST` | `/api/v1/transactions` | Create transaction (idempotent) |
| `GET` | `/api/v1/transactions/:id` | Fetch by ID |
| `GET` | `/api/v1/transactions` | List all (optional `?status=&currency=` filters) |
| `PATCH` | `/api/v1/transactions/:id/status` | Update status (enforced transitions) |

### Example

```bash
curl -X POST http://localhost:3000/api/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{
    "idempotency_key": "inv-2024-001",
    "amount": 250.00,
    "currency": "USD",
    "description": "Invoice payment"
  }'
```

## Running

```bash
cargo run              # starts on :8080 by default
PORT=3000 cargo run    # override with PORT env var
cargo test             # 21 tests (unit + integration)
cargo clippy -- -D warnings
```
