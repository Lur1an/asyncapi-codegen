# AsyncAPI Schema Parser & Code Generator
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
## Sidenote for Rust users
- For `OneOf` schemas with a specific `discriminator` set it currently only works if the discriminator matches the name of the entity (For anonymous entity set the `title` property for a deterministic name), otherwise if you use special values for the discriminator inside of `const` fields you need to omit the `discriminator` for now and just use the `#[serde(untagged)]` enum that is generated, `const` fields will be respected through the use of the `monostate` crate.
- `AllOf` schemas currently don't merge properties, out of lazyness they create struct for inner schemas and then put them in a single struct through `#[serde(flatten)]`. (Out of simplicity I may use a solution like this in other languages, having a named empty class inherit from anonymous/named structs for its fields)
## Planned
- A CLI tool for code generation
- Python `pydantic` model generator
- A protobuf generator

## Issues
The `deserializer` defines `untagged` enums with `monostate::MustBe` for the deserialization of a schema, this leads to quite unhelpful error messages when you schema does not match, most of the errors are `Did not match any variant in SchemaDef`

