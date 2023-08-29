# AsyncAPI Schema Parser
A language agnostic model generator for AsyncAPI schemas. (Rust only for now, next planned: python + pydantic)

## How to Use
Check out the `codegen-test` crate to see how it can be used to generate your code at compile time.

## Schema constraints
While writing the `parser` and `deserializer` I put some constraints of possible schemas, this only allows a subset of possible `asyncapi` schema definitions.
- Every top-level item in the `components -> schemas` part of the document needs to be an actual schema, this would not be allowed:
```yaml
components:
  schemas:
    MyEntity:
      $ref: '#/components/schemas/AnotherEntity'
```
- Every top-level schema can only be one of the following: `[AllOf, OneOf, AnyOf, type: object]`, top-level `array` types don't currently work, in your asyncapi schema I'd recommend creating an anonymous schema in the `messages` part
of the specification and creating the specific `item` type for the `array` items in the `components/schemas` section such that your code will have the type for the items and you can easily deserialize payloads by wrapping it in language specific collections.
- Currently enums only work with String values, even if they're supported at deserialization/parsing at generation time numerical enums will throw errors as I haven't created a specific type to distinguish them from Literal enums.
- Every time a `const` value is specified there must be a `type` with it.
- Currently only integers are supported and any `format` directive is simply ignored

## Planned
- A CLI tool for code generation
- A protobuf generator

