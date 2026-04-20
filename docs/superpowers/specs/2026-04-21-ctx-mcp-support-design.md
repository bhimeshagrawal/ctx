# ctx MCP Support Design

Date: 2026-04-21
Status: Proposed

## Summary

Add MCP support to the Rust `ctx` codebase without turning `ctx` into a hosted platform.

The first MCP release should:

- support both `stdio` and HTTP/SSE transports
- prioritize agent clients such as Codex and Claude
- expose `tools`, `resources`, and `prompts`
- aim for near-full CLI parity
- treat the MCP contract as public and stable

The recommended architecture is a shared Rust service core with thin CLI and MCP front doors. The CLI stays first-class. MCP becomes a second interface over the same operations.

## Goals

- Add MCP support without duplicating core business logic.
- Keep CLI and MCP behavior aligned by routing both through shared services.
- Support `stdio` and HTTP/SSE in the first MCP release.
- Expose a stable MCP contract for tools, resources, and prompts.
- Preserve `ctx`'s local-first model, local storage, and single-user runtime.
- Keep HTTP/SSE local-only by default.

## Non-Goals

- Build a hosted multi-user memory service.
- Add remote auth, multi-tenant isolation, or internet-facing deployment support in V1.
- Replace the CLI with MCP.
- Turn prompts into hidden mutation flows.
- Freeze internal Rust module boundaries if a small refactor improves the public contract.

## Recommended Approach

Three approaches were considered:

1. Shared service core with transport adapters.
2. Thin MCP wrapper over existing CLI handlers.
3. Separate daemon-oriented server domain.

The recommended approach is `1`.

It requires more refactoring than a thin wrapper, but it is the only option that keeps CLI and MCP semantics aligned, avoids duplicated business logic, and gives the project a stable public contract.

## Architecture

The MCP design should have five layers:

1. CLI adapter
2. MCP server core
3. Application services
4. Storage and retrieval
5. System integration

### CLI Adapter

The existing CLI remains intact. `clap` continues to own user-facing parsing and help text, but command modules become thin adapters from CLI args into typed service requests.

### MCP Server Core

The MCP core owns capability advertisement, tool registration, resource registration, prompt registration, schema validation, and request dispatch. It is transport-neutral.

### Application Services

Application services implement the actual behavior for setup, diagnostics, config inspection, memory ingest, memory retrieval, update, and uninstall. Services return typed results and structured errors. They do not render terminal output and do not depend on `clap` or MCP transport details.

### Storage And Retrieval

The existing local data model remains the source of truth. Services continue to use the current path resolution, config loading, embedding provider, chunking, LanceDB storage, and ranking logic.

### System Integration

System integration owns path resolution, environment discovery, model initialization, database bootstrap, HTTP binding, and destructive side-effect execution.

## Transport Design

The first MCP release should support two transports:

- `stdio`
- HTTP/SSE

Both transports must share the same MCP core and service layer.

### `stdio`

`stdio` is the default transport for agent clients that spawn `ctx` directly. It should be exposed through a command such as `ctx mcp serve --transport stdio`.

### HTTP/SSE

HTTP/SSE should be exposed through the same command family, for example `ctx mcp serve --transport http --host 127.0.0.1 --port 0`.

HTTP/SSE must:

- bind to `127.0.0.1` by default
- target same-machine clients only in V1
- avoid auth in V1
- document clearly that remote exposure is unsupported

## Shared Service Layer

The current command handlers own too much orchestration logic for MCP to reuse them directly. The project should extract a service layer with explicit request and response types.

The first service modules should cover:

- `setup`
- `doctor`
- `config show`
- `memory add`
- `memory search`
- `update`
- `uninstall`

Each service should:

- accept a request struct, not CLI flags
- return a typed response struct
- return structured domain errors
- avoid terminal rendering
- avoid MCP transport concerns

This extraction lets the CLI and MCP stay behaviorally consistent.

## MCP Surface

The first MCP release should expose `tools`, `resources`, and `prompts`.

### Tools

Tools should cover command-like operations, including mutations.

Initial tool set:

- `memory_add`
- `memory_search`
- `setup_run`
- `doctor_run`
- `config_show`
- `update_run`
- `uninstall_run`

Tool names should be treated as public contract. They should not mirror CLI syntax exactly, because CLI flags may evolve faster than MCP schemas.

### Resources

Resources should expose read-only state that agent clients may want to inspect without invoking mutations.

Initial resource candidates:

- effective config
- resolved managed paths
- runtime status
- embedding model metadata
- selected memory or document metadata if the implementation can expose it without inventing a second query model

### Prompts

Prompts should guide common workflows without performing mutations by themselves.

Initial prompt candidates:

- ingest text into memory
- ingest file into memory
- search memory with filters
- first-time setup
- safe uninstall
- self-update

Prompts remain advisory. Actual state changes happen only through tools.

## Request And Response Design

The MCP contract should use structured requests and responses, not CLI-shaped flag emulation.

### `memory_add`

Instead of mutually exclusive flags, `memory_add` should accept an explicit source object. Example shapes:

- `{"source":{"kind":"text","text":"..."}}`
- `{"source":{"kind":"file","path":"..."}}`

This keeps the MCP contract clear and avoids leaking CLI parsing rules into the public API.

### `memory_search`

`memory_search` should return a stable result envelope that includes:

- original query
- applied filters
- top-k value
- ranked hits

The current CLI JSON output is a useful starting point, but MCP response fields should be formalized now and then treated as stable.

## Lifecycle And Runtime

Both CLI and MCP should enter through a shared bootstrap that resolves paths, loads config, initializes the embedding provider, and opens the database.

The MCP server runtime should cache reusable process state where safe. That matters most for HTTP/SSE, where repeated requests would otherwise reinitialize the embedding runtime and database connections too often.

`ctx` should remain a single-process, local-first tool. MCP support does not require an always-on daemon architecture.

## Safety Model

The MCP surface should expose destructive operations by default because the product goal is near-full CLI parity and the client is expected to handle confirmation.

That choice requires explicit machine-readable risk signals.

Tools such as `update_run` and `uninstall_run` should make their side effects obvious in their schema and results. Agent clients must be able to detect that a call mutates local state or deletes data.

The server should not hide destructive behavior behind prompts or undocumented fields.

## Error Model

Define a stable set of domain errors before wiring transports.

Initial error categories:

- invalid input
- missing file
- configuration error
- embedding initialization failure
- storage failure
- unsupported operation
- destructive-operation warning

CLI rendering can stay human-oriented. MCP errors must map to predictable codes and payload fields so clients can recover programmatically.

## Validation Rules

Validation should happen before side effects.

Examples:

- `memory_add` source validation
- file existence checks
- chunk-size and chunk-overlap rules
- empty search query rejection
- argument validation for setup, update, and uninstall flows

MCP schemas should reject malformed requests early, but services must still enforce invariants so CLI and MCP cannot drift apart.

## Compatibility Contract

MCP V1 should be treated as a public contract.

That means:

- tool names stay stable
- resource identifiers stay stable
- prompt identifiers stay stable
- field names and meanings stay stable
- serialized request and response shapes are tested directly

Internal refactors remain allowed as long as the public MCP contract does not break.

## Testing Strategy

The MCP implementation should ship with five test layers:

1. unit tests for service requests, responses, and validation
2. integration tests for ingest and search against the local database and embedding boundary
3. MCP transport tests for `stdio`
4. MCP transport tests for HTTP/SSE
5. contract tests that snapshot the serialized MCP shapes for core tools, resources, prompts, and errors

The project should also add regression tests that prove CLI and MCP produce equivalent service outcomes for the same inputs.

## Implementation Sequence

1. Extract shared service functions from the current CLI command handlers.
2. Define stable request, response, and error types for the public MCP surface.
3. Add the transport-neutral MCP core and register tools, resources, and prompts.
4. Wire `stdio` transport.
5. Wire HTTP/SSE transport with localhost-only defaults.
6. Add contract tests, transport tests, and CLI-vs-MCP regression tests.
7. Document the MCP command surface, local-only HTTP/SSE security stance, and compatibility guarantees.

## Risks

- A thin-wrapper implementation would leak CLI parsing and output concerns into MCP.
- Public-stable MCP schemas raise the cost of renaming fields later.
- Exposing destructive operations by default increases the importance of explicit risk metadata.
- HTTP/SSE without auth is acceptable only because V1 binds locally and targets same-machine agent clients.

## Open Questions

- Which Rust MCP library best fits the desired public-stable contract and dual-transport shape.
- Whether memory metadata belongs in resources, additional tools, or both.
- How much bootstrap state can be cached safely across long-lived HTTP/SSE sessions.
