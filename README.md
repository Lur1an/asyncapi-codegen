# AsyncAPI Schema Parser
---
A language agnostic model generator for AsyncAPI schemas. (Rust only for now, next planned: python + pydantic)

## How to Use
---
Check out the `codegen-test` crate to see how it can be used to generate your code at compile time.

## Schema constraints
---
While writing the `parser` and `deserializer` I put some constraints of possible schemas, this only allows a subset of possible `asyncapi` schema definitions.

