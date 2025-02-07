# Chain

A chain is a intermediate data type to enable complex template values. Chains also provide additional customization, such as marking values as sensitive.

To use a chain in a template, reference it as `{{chains.<id>}}`.

## Fields

| Field          | Type                                                                                   | Description                                                                                                                            | Default  |
| -------------- | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | -------- |
| `source`       | [`ChainSource`](./chain_source.md)                                                     | Source of the chained value                                                                                                            | Required |
| `sensitive`    | `boolean`                                                                              | Should the value be hidden in the UI?                                                                                                  | `false`  |
| `selector`     | [`JSONPath`](https://www.ietf.org/archive/id/draft-goessner-dispatch-jsonpath-00.html) | Selector to transform/narrow down results in a chained value. See [Filtering & Querying](../../user_guide/filter_query.md)             | `null`   |
| `content_type` | [`ContentType`](./content_type.md)                                                     | Force content type. Not required for `request` and `file` chains, as long as the `Content-Type` header/file extension matches the data |          |

See the [`ChainSource`](./chain_source.md) docs for detail on the different types of chainable values.

## Examples

```yaml
# Load chained value from a file
username:
  source: !file
    path: ./username.txt
---
# Prompt the user for a value whenever the request is made
password:
  source: !prompt
    message: Enter Password
  sensitive: true
---
# Use a value from another response
# Assume the request recipe with ID `login` returns a body like `{"token": "foo"}`
auth_token:
  source: !request
    recipe: login
  selector: $.token
```
