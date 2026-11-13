# Invalid examples

Every file here must be rejected by its schema and by the types of every language (contract test 4, see [doc/interfaces.md](../../../doc/interfaces.md#language-neutral-contract)).

File name: `<program or common>/<schema name>.<case>.json`. Each file names the error it expects, starting with the field the error is about (`"<field>: <reason>"`, checked by LIB-CON-03 in [doc/testing/shared_libraries.md](../../../doc/testing/shared_libraries.md#contract-schemas-and-examples-contract)):
```json
{
  "expected_error": "request_id: missing",
  "message": { "type": "ping", "version": 1, "payload": {} }
}
```
