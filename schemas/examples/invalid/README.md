# Invalid examples

Every file here must be rejected by its schema and by the types of every language (contract test 4, see [doc/interfaces.md](../../../doc/interfaces.md#language-neutral-contract)).

File name: `<program or common>/<schema name>.<case>.json`. Each file names the error it expects:
```json
{
  "expected_error": "missing required field request_id",
  "message": { "type": "ping", "version": 1, "payload": {} }
}
```
