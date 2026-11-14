# schemas

The language-neutral contract between all TargetScope programs. If [doc/interfaces.md](../doc/interfaces.md) and a schema differ, the schema wins and the document is fixed.

```
schemas/
├── common/            # envelopes, error codes, shared types (Session, Shot, Device, TargetType, …), ping
├── <program>/         # one file per request, response and event of that program
├── export/v1.schema.json   # export/import file format
├── examples/          # one valid example per message schema, used by the contract tests
│   ├── <program>/     #   same names as the schemas
│   └── invalid/       #   examples that must be rejected, each naming its expected error
└── frames/frame_header.h   # iceoryx2 frame header
```

- JSON Schema draft 2020-12. `$id`s use the reserved host `targetscope.invalid`; references between schemas are relative.
- Message schemas combine an envelope (`common/*_envelope.schema.json`) with the message's `type` and its payload in `$defs/payload`.
- Every schema sets `additionalProperties: false` and the value ranges of its fields; `description` links the section of `doc/interfaces.md` that explains it.
- `examples/common/` has one example per shared type and per envelope. Number fields that are floats in the types use values with a fraction (e.g. `200.5`), so the round trip can't turn `200` into `200.0` unnoticed.
