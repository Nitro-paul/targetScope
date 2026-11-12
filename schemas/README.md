# schemas

The language-neutral contract between all TargetScope programs. If [doc/interfaces.md](../doc/interfaces.md) and a schema differ, the schema wins and the document is fixed.

```
schemas/
├── common/            # envelopes, error codes, shared types (Session, Shot, Device, TargetType, …), ping
├── <program>/         # one file per request, response and event of that program
├── export/v1.schema.json   # export/import file format
├── examples/          # one valid example per message schema, used by the contract tests
│   ├── <program>/     #   same names as the schemas
│   ├── target_types/  #   target types with their expected rings (shared by Rust, C++ and JS tests)
│   └── invalid/       #   examples that must be rejected, each naming its expected error
└── frames/frame_header.h   # iceoryx2 frame header
```

- JSON Schema draft 2020-12. `$id`s use the reserved host `targetscope.invalid`; references between schemas are relative.
- Message schemas combine an envelope (`common/*_envelope.schema.json`) with the message's `type` and its payload in `$defs/payload`.
- Payloads marked with a `$comment` "TODO" are skeletons; their fields are described in the linked section of `doc/interfaces.md`. The generated examples in `examples/` have empty payloads and must be completed together with their schema.
