# Fixtures

Specs used by oadig integration tests and manual exploration.

## Hand-written (this project's license)

- `petstore.yaml` / `petstore.json` — minimal OpenAPI 3.1 petstore.
- `circular.yaml` — exercises a circular `$ref` (Node → children → Node).
- `swagger2.yaml` — tiny Swagger 2.0 spec for legacy-format regression tests.

## Third-party (Apache License 2.0)

Everything under `oai/` is copied verbatim (or close to it) from the
[OpenAPI-Specification examples](https://github.com/OAI/OpenAPI-Specification)
at tags `3.0.3` and `3.1.0`:

| File | OpenAPI version | What it exercises |
|---|---|---|
| `oai/petstore-expanded.yaml` | 3.0.0 | Full CRUD, parameters, requestBody, multiple responses, errors, $ref |
| `oai/api-with-examples.yaml` | 3.0.0 | The `examples` keyword on media types |
| `oai/callback-example.yaml` | 3.0.0 | The `callbacks` feature |
| `oai/link-example.yaml` | 3.0.0 | Response `links` |
| `oai/uspto.yaml` | 3.0.1 | Realistic API with complex schemas |
| `oai/webhook-example.yaml` | 3.1.0 | 3.1-only `webhooks` |

License: Apache 2.0 (same as the upstream repository). Keep any upstream
attribution intact when modifying.
