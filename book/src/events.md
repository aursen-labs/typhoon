# Events

Events let a program record structured data in the transaction logs so that
off-chain clients and indexers can react to what happened on-chain. Typhoon
events are zero-copy: emitting one writes the event's discriminator followed by
its packed fields with the `sol_log_data` syscall — no heap allocation, no
serialization pass.

Events are also written into the program's generated IDL, so clients can decode
the log payload against a known schema.

## Defining events

Annotate an enum with `#[event]`. Each variant is an event, and its
discriminator is the index of the variant encoded as a single `u8`:

```rust
use typhoon::prelude::*;

#[event]
pub enum CounterEvent {
    Incremented { count: u64 },   // discriminator 0
    Decremented { count: u64 },   // discriminator 1
}
```

Variant fields must be plain-old-data (`bytemuck::NoUninit`). The attribute
generates an `Emit` implementation, which provides the `.emit()` method.

## Emitting an event

Construct a variant and call `.emit()` on it:

```rust
pub fn increment(mut ctx: CounterMut) -> ProgramResult {
    let count = {
        let mut data = ctx.counter.mut_data()?;
        data.count += 1;
        data.count
    };

    CounterEvent::Incremented { count }.emit();

    Ok(())
}
```

`emit()` writes a single `Program data:` log field whose bytes are the
discriminator (the variant index) followed by the packed fields. Off-chain, the
base64 payload can be matched against the `events` entry in the IDL — read the
first byte to pick the variant, then decode the fields.

## In the IDL

Every variant of an `#[event]` enum appears under the `events` array of the
generated IDL, with its field layout and its one-byte discriminator:

```json
{
  "kind": "eventNode",
  "name": "incremented",
  "data": {
    "kind": "structTypeNode",
    "fields": [
      { "kind": "structFieldTypeNode", "name": "count", "type": { "kind": "numberTypeNode", "format": "u64", "endian": "le" } }
    ]
  },
  "discriminators": [
    { "kind": "constantDiscriminatorNode", "offset": 0, "constant": { "kind": "constantValueNode", "type": { "kind": "bytesTypeNode" }, "value": { "kind": "bytesValueNode", "data": "AA==", "encoding": "base64" } } }
  ]
}
```

The discriminator value `"AA=="` is the base64 encoding of the single byte `0` —
the index of the `Incremented` variant.

## Runnable example

See [`examples/events`](https://github.com/exotic-markets-labs/typhoon/tree/main/examples/events)
for a complete counter program that emits a `CounterEvent::Incremented` event,
with an integration test that asserts the event appears in the logs and in the IDL.
