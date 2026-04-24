use std::process::Command;

use serde_json::{Value, json};

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_oadig"))
}

fn run_json(args: &[&str]) -> Value {
    let out = bin().args(args).output().expect("spawn oadig");
    assert!(
        out.status.success(),
        "command failed: {:?}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("stdout is JSON")
}

const PETSTORE_YAML: &str = "tests/fixtures/petstore.yaml";
const PETSTORE_JSON: &str = "tests/fixtures/petstore.json";
const CIRCULAR_YAML: &str = "tests/fixtures/circular.yaml";
const SWAGGER2_YAML: &str = "tests/fixtures/swagger2.yaml";

#[test]
fn spec_returns_openapi_for_v3() {
    let v = run_json(&["spec", PETSTORE_YAML]);
    assert_eq!(v, "3.1.0");
}

#[test]
fn spec_returns_swagger_for_v2() {
    let v = run_json(&["spec", SWAGGER2_YAML]);
    assert_eq!(v, "2.0");
}

#[test]
fn spec_is_null_when_missing() {
    let out = bin()
        .args(["spec", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    use std::io::Write;
    out.stdin
        .as_ref()
        .unwrap()
        .write_all(b"info:\n  title: Orphan\n  version: \"0\"\n")
        .unwrap();
    let result = out.wait_with_output().unwrap();
    assert!(result.status.success());
    let v: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(v.is_null());
}

#[test]
fn info_show_null_fills_missing_info_keys() {
    let out = bin()
        .args(["info", "-", "--show-null"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    use std::io::Write;
    out.stdin
        .as_ref()
        .unwrap()
        .write_all(b"info:\n  title: Orphan\n  version: \"0\"\n")
        .unwrap();
    let result = out.wait_with_output().unwrap();
    assert!(result.status.success());
    let value: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(value["description"].is_null());
    assert!(value["contact"].is_null());
    assert!(value["license"].is_null());
    // openapi/swagger/servers live on the top-level spec, not the info object.
    assert!(value.get("openapi").is_none());
    assert!(value.get("servers").is_none());
}

#[test]
fn info_omits_spec_version_and_servers() {
    let info = run_json(&["info", PETSTORE_YAML]);
    assert!(info.get("openapi").is_none());
    assert!(info.get("swagger").is_none());
    assert!(info.get("servers").is_none());
    assert_eq!(info["title"], "Petstore");
}

#[test]
fn info_yaml_and_json_match() {
    let from_yaml = run_json(&["info", PETSTORE_YAML]);
    let from_json = run_json(&["info", PETSTORE_JSON]);
    assert_eq!(from_yaml, from_json);
    assert_eq!(from_yaml["title"], "Petstore");
    assert_eq!(from_yaml["version"], "1.0.0");
}

#[test]
fn stats_counts_match_fixture() {
    let stats = run_json(&["stats", PETSTORE_YAML]);
    assert_eq!(stats["paths"], 2);
    assert_eq!(stats["operations"], 3);
    assert_eq!(stats["schemas"], 2);
    assert_eq!(stats["by_method"]["GET"], 2);
    assert_eq!(stats["by_method"]["POST"], 1);
    assert_eq!(stats["by_tag"]["pets"], 3);
}

#[test]
fn paths_lists_unique_path_strings() {
    let paths = run_json(&["paths", PETSTORE_YAML]);
    assert_eq!(paths, json!(["/pets", "/pets/{petId}"]));
}

#[test]
fn paths_filter_by_path_prefix() {
    let paths = run_json(&["paths", PETSTORE_YAML, "--filter", "path=/pets/*"]);
    assert_eq!(paths, json!(["/pets/{petId}"]));
}

#[test]
fn paths_filter_rejects_non_path_key() {
    let out = bin()
        .args(["paths", PETSTORE_YAML, "--filter", "method=GET"])
        .output()
        .expect("spawn");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("path"));
}

#[test]
fn operations_filter_method_exact() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--filter", "method=POST"]);
    let arr = ops.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["method"], "POST");
}

#[test]
fn operations_filter_method_list_or() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--filter", "method=GET,POST"]);
    assert_eq!(ops.as_array().unwrap().len(), 3);
}

#[test]
fn operations_filter_path_contains() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--filter", "path=*petId*"]);
    let arr = ops.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["path"], "/pets/{petId}");
}

#[test]
fn operations_filter_path_prefix() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--filter", "path=/pets/*"]);
    let arr = ops.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["path"], "/pets/{petId}");
}

#[test]
fn operations_filter_combines_as_and() {
    let ops = run_json(&[
        "operations",
        PETSTORE_YAML,
        "--filter",
        "tag=pets",
        "--filter",
        "method=POST",
    ]);
    let arr = ops.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["method"], "POST");
}

#[test]
fn operations_filter_tag_exact() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--filter", "tag=pets"]);
    assert_eq!(ops.as_array().unwrap().len(), 3);
    let none = run_json(&["operations", PETSTORE_YAML, "--filter", "tag=nonexistent"]);
    assert!(none.as_array().unwrap().is_empty());
}

#[test]
fn operations_filter_rejects_unknown_key() {
    let out = bin()
        .args(["operations", PETSTORE_YAML, "--filter", "foo=bar"])
        .output()
        .expect("spawn");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("unknown filter key"));
}

#[test]
fn operations_default_leads_with_operation_id() {
    let ops = run_json(&["operations", PETSTORE_YAML]);
    let expected = json!([
        { "operationId": "listPets",    "method": "GET",  "path": "/pets",         "summary": "List all pets" },
        { "operationId": "createPet",   "method": "POST", "path": "/pets",         "summary": "Create a pet" },
        { "operationId": "showPetById", "method": "GET",  "path": "/pets/{petId}", "summary": "Info for a specific pet" },
    ]);
    assert_eq!(ops, expected);
}

#[test]
fn operations_include_adds_tags_and_operation_id() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--include", "tags,operationId"]);
    let first = &ops[0];
    assert_eq!(first["method"], "GET");
    assert_eq!(first["path"], "/pets");
    assert_eq!(first["summary"], "List all pets");
    assert_eq!(first["tags"], json!(["pets"]));
    assert_eq!(first["operationId"], "listPets");
}

#[test]
fn operations_exclude_summary_drops_default_field() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--exclude", "summary"]);
    for op in ops.as_array().unwrap() {
        assert!(op.get("summary").is_none(), "summary should be excluded");
        assert!(op["method"].is_string());
        assert!(op["path"].is_string());
    }
}

#[test]
fn operations_include_all_covers_known_fields() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--include", "all"]);
    // GET /pets has parameters in the fixture
    let get_pets = ops
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["method"] == "GET" && o["path"] == "/pets")
        .unwrap();
    assert_eq!(get_pets["tags"], json!(["pets"]));
    assert_eq!(get_pets["operationId"], "listPets");
    assert!(get_pets["parameters"].is_array());
}

#[test]
fn operations_include_response_resolves_refs() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--include", "response"]);
    let get_pets = ops
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["method"] == "GET" && o["path"] == "/pets")
        .unwrap();
    // response should be the responses map, with $ref to Pets resolved inline.
    let schema = &get_pets["responses"]["200"]["content"]["application/json"]["schema"];
    assert!(schema.get("$ref").is_none(), "ref should be resolved");
    assert_eq!(schema["type"], "array");
}

#[test]
fn ops_alias_resolves_to_operations() {
    let from_alias = run_json(&["ops", PETSTORE_YAML]);
    let from_canonical = run_json(&["operations", PETSTORE_YAML]);
    assert_eq!(from_alias, from_canonical);
}

#[test]
fn operation_lookup_by_id_shows_full_detail() {
    let op = run_json(&["operation", "listPets", PETSTORE_YAML]);
    assert_eq!(op["method"], "GET");
    assert_eq!(op["path"], "/pets");
    assert_eq!(op["operationId"], "listPets");
    let schema = &op["responses"]["200"]["content"]["application/json"]["schema"];
    assert!(schema.get("$ref").is_none());
    assert_eq!(schema["type"], "array");
}

#[test]
fn operation_lookup_by_method_and_path_matches_id() {
    let by_id = run_json(&["operation", "listPets", PETSTORE_YAML]);
    let by_mp = run_json(&["operation", PETSTORE_YAML, "-m", "GET", "-p", "/pets"]);
    assert_eq!(by_id, by_mp);
}

#[test]
fn op_alias_resolves_to_operation() {
    let from_alias = run_json(&["op", "listPets", PETSTORE_YAML]);
    let from_canonical = run_json(&["operation", "listPets", PETSTORE_YAML]);
    assert_eq!(from_alias, from_canonical);
}

#[test]
fn operation_method_is_case_insensitive() {
    let upper = run_json(&["operation", PETSTORE_YAML, "-m", "GET", "-p", "/pets"]);
    let lower = run_json(&["operation", PETSTORE_YAML, "-m", "get", "-p", "/pets"]);
    assert_eq!(upper, lower);
}

#[test]
fn operation_id_not_found_errors() {
    let out = bin()
        .args(["operation", "nonexistent", PETSTORE_YAML])
        .output()
        .expect("spawn");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("nonexistent"));
    assert!(stderr.contains("hint"));
}

#[test]
fn operation_method_path_not_found_errors() {
    let out = bin()
        .args(["operation", PETSTORE_YAML, "-m", "DELETE", "-p", "/pets"])
        .output()
        .expect("spawn");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("DELETE"));
    assert!(stderr.contains("/pets"));
}

#[test]
fn operation_requires_either_id_or_method_path() {
    let out = bin()
        .args(["operation", PETSTORE_YAML])
        .output()
        .expect("spawn");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("specify either"));
}

#[test]
fn response_narrows_by_status() {
    let all = run_json(&["response", "listPets", PETSTORE_YAML]);
    assert!(all.get("200").is_some());
    let only_200 = run_json(&["response", "listPets", PETSTORE_YAML, "--status", "200"]);
    assert_eq!(only_200, all["200"]);
}

#[test]
fn res_alias_resolves_to_response() {
    let from_alias = run_json(&["res", "listPets", PETSTORE_YAML]);
    let from_canonical = run_json(&["response", "listPets", PETSTORE_YAML]);
    assert_eq!(from_alias, from_canonical);
}

#[test]
fn request_returns_null_when_missing() {
    // petstore listPets has no requestBody
    let req = run_json(&["request", "listPets", PETSTORE_YAML]);
    assert!(req.is_null());
}

#[test]
fn requests_lists_only_operations_with_body() {
    let reqs = run_json(&["requests", PETSTORE_YAML]);
    let arr = reqs.as_array().unwrap();
    // Only POST /pets has a requestBody in the fixture.
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["method"], "POST");
    assert_eq!(arr[0]["path"], "/pets");
    // $ref to Pet should be inlined.
    let schema = &arr[0]["requestBody"]["content"]["application/json"]["schema"];
    assert!(schema.get("$ref").is_none());
    assert_eq!(schema["type"], "object");
}

#[test]
fn statuses_dedupes_by_code() {
    let v = run_json(&["statuses", PETSTORE_YAML]);
    assert_eq!(
        v,
        json!([
            { "status": "200", "description": "A paged array of pets" },
            { "status": "201", "description": "Null response" }
        ])
    );
}

#[test]
fn schemas_lists_definitions_from_swagger2() {
    // swagger2-with-ref.yaml defines `Thing` under top-level `definitions`.
    // Auto-convert should surface it via components.schemas.
    let v = run_json(&["schemas", "tests/fixtures/swagger2-with-ref.yaml"]);
    assert_eq!(v, json!(["Thing"]));
}

#[test]
fn schema_resolves_swagger2_definition_by_name() {
    let v = run_json(&["schema", "Thing", "tests/fixtures/swagger2-with-ref.yaml"]);
    assert_eq!(v["type"], "object");
}

#[test]
fn operations_on_swagger2_lists_by_converted_shape() {
    let v = run_json(&["operations", SWAGGER2_YAML]);
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["method"], "GET");
    assert_eq!(arr[0]["path"], "/ping");
    assert_eq!(arr[0]["operationId"], "ping");
}

#[test]
fn statuses_include_schema_for_swagger2_uses_application_json_after_convert() {
    let v = run_json(&["statuses", SWAGGER2_YAML, "--include", "schema"]);
    let arr = v.as_array().unwrap();
    let first = arr.iter().find(|e| e["status"] == "200").unwrap();
    let schema = &first["schema"]["application/json"];
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["ok"]["type"], "boolean");
}

#[test]
fn statuses_include_content_returns_full_content_map() {
    let v = run_json(&["statuses", PETSTORE_YAML, "--include", "content"]);
    let arr = v.as_array().unwrap();
    let first = arr.iter().find(|e| e["status"] == "200").unwrap();
    let media = &first["content"]["application/json"];
    // content includes media-type level fields beyond schema
    assert!(media.get("schema").is_some());
}

#[test]
fn statuses_include_schema_resolves_refs() {
    let v = run_json(&["statuses", PETSTORE_YAML, "--include", "schema"]);
    let arr = v.as_array().unwrap();
    let first = arr
        .iter()
        .find(|e| e["status"] == "200")
        .expect("200 entry");
    let schema = &first["schema"]["application/json"];
    assert!(schema.get("$ref").is_none(), "ref should be inlined");
    assert_eq!(schema["type"], "array");
}

#[test]
fn statuses_include_skips_absent_fields() {
    let v = run_json(&["statuses", PETSTORE_YAML, "--include", "headers,schema"]);
    let arr = v.as_array().unwrap();
    let only_desc = arr
        .iter()
        .find(|e| e["status"] == "201")
        .expect("201 entry");
    // 201 has no content and no headers in the fixture; neither key should appear.
    assert!(only_desc.get("headers").is_none());
    assert!(only_desc.get("schema").is_none());
}

#[test]
fn responses_lists_every_operation() {
    let res = run_json(&["responses", PETSTORE_YAML]);
    let arr = res.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    for entry in arr {
        assert!(entry["responses"].is_object());
    }
}

#[test]
fn responses_narrows_by_status_and_drops_misses() {
    let res = run_json(&["responses", PETSTORE_YAML, "--status", "201"]);
    let arr = res.as_array().unwrap();
    // Only POST /pets has a 201 in the fixture.
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["method"], "POST");
    assert_eq!(arr[0]["path"], "/pets");
    assert!(arr[0]["responses"]["201"].is_object());
    assert!(arr[0]["responses"].get("200").is_none());
}

#[test]
fn operations_lines_format_one_entry_per_line() {
    let out = bin()
        .args(["operations", PETSTORE_YAML, "--lines"])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("[\n  {"));
    assert!(stdout.trim_end().ends_with("}\n]"));
    for line in stdout.lines().filter(|l| l.starts_with("  {")) {
        assert!(line.contains("\"method\""));
        assert!(line.contains("\"path\""));
    }
}

#[test]
fn search_finds_substring_case_insensitive_by_default() {
    let hits = run_json(&["search", "petstore", PETSTORE_YAML]);
    let arr = hits.as_array().unwrap();
    assert!(!arr.is_empty());
    // info/title contains "Petstore" — case-insensitive substring should hit.
    assert!(arr.iter().any(|h| h["pointer"] == "/info/title"));
}

#[test]
fn search_default_includes_operation_context() {
    let hits = run_json(&["search", "List all pets", PETSTORE_YAML]);
    let arr = hits.as_array().unwrap();
    assert!(!arr.is_empty());
    let h = &arr[0];
    assert_eq!(h["operationRef"]["method"], "GET");
    assert_eq!(h["operationRef"]["path"], "/pets");
    assert_eq!(h["at"], "summary");
    assert_eq!(h["value"], "List all pets");
    // path is opt-in, not in default set
    assert!(h.get("path").is_none());
}

#[test]
fn search_include_jsonpath_renders_dotted_with_bracket_quoting() {
    let hits = run_json(&["search", "Petstore", PETSTORE_YAML, "--include", "jsonPath"]);
    let info_hit = hits
        .as_array()
        .unwrap()
        .iter()
        .find(|h| h["pointer"] == "/info/title")
        .unwrap();
    assert_eq!(info_hit["jsonPath"], "$.info.title");
}

#[test]
fn search_jsonpath_quotes_complex_keys() {
    let hits = run_json(&[
        "search",
        "Info for a specific pet",
        PETSTORE_YAML,
        "--include",
        "jsonPath",
    ]);
    let h = &hits.as_array().unwrap()[0];
    assert_eq!(h["jsonPath"], "$.paths[\"/pets/{petId}\"].get.summary");
}

#[test]
fn search_exclude_pointer_drops_field() {
    let hits = run_json(&["search", "petstore", PETSTORE_YAML, "--exclude", "pointer"]);
    for h in hits.as_array().unwrap() {
        assert!(h.get("pointer").is_none());
        assert!(h["value"].is_string());
    }
}

#[test]
fn search_hits_outside_operations_have_no_context() {
    let hits = run_json(&["search", "Pet", PETSTORE_YAML]);
    let components_hit = hits
        .as_array()
        .unwrap()
        .iter()
        .find(|h| {
            h["pointer"]
                .as_str()
                .is_some_and(|p| p.starts_with("/components/"))
        })
        .expect("at least one components hit");
    assert!(components_hit.get("operationRef").is_none());
    assert!(components_hit.get("at").is_none());
}

#[test]
fn search_case_sensitive_skips_case_mismatch() {
    let insensitive = run_json(&["search", "petstore", PETSTORE_YAML]);
    let sensitive = run_json(&["search", "petstore", PETSTORE_YAML, "--case-sensitive"]);
    // "Petstore" matches in insensitive mode but not in sensitive mode.
    let insensitive_titles: Vec<_> = insensitive
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["pointer"] == "/info/title")
        .collect();
    let sensitive_titles: Vec<_> = sensitive
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["pointer"] == "/info/title")
        .collect();
    assert_eq!(insensitive_titles.len(), 1);
    assert!(sensitive_titles.is_empty());
}

#[test]
fn search_regex_mode() {
    let hits = run_json(&[
        "search",
        r"^List all pets$",
        PETSTORE_YAML,
        "--regex",
        "--case-sensitive",
    ]);
    let arr = hits.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["pointer"], "/paths/~1pets/get/summary");
}

#[test]
fn convert_swagger2_to_openapi30_round_trips_through_validate() {
    let bin_path = env!("CARGO_BIN_EXE_oadig");
    let converted = Command::new(bin_path)
        .args(["convert", "3.0", SWAGGER2_YAML])
        .output()
        .expect("spawn convert");
    assert!(converted.status.success());

    let value: Value = serde_json::from_slice(&converted.stdout).unwrap();
    assert_eq!(value["openapi"], "3.0.3");
    assert_eq!(value["servers"][0]["url"], "https://api.example.com/v1");
    assert!(
        value["paths"]["/ping"]["get"]["responses"]["200"]["content"]["application/json"]["schema"]
            .is_object()
    );

    // Pipe through validate; expect valid.
    let validator = Command::new(bin_path)
        .args(["validate", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn validate");
    use std::io::Write;
    validator
        .stdin
        .as_ref()
        .unwrap()
        .write_all(&converted.stdout)
        .unwrap();
    let result = validator.wait_with_output().unwrap();
    assert!(result.status.success());
    let verdict: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(verdict["valid"], true);
}

#[test]
fn convert_preserves_top_level_vendor_extensions() {
    let v = run_json(&["convert", "3.0", SWAGGER2_YAML]);
    assert_eq!(v["x-vendor-tag"], "legacy-fixture");
}

#[test]
fn convert_reshapes_oauth2_security_scheme() {
    let v = run_json(&["convert", "3.0", SWAGGER2_YAML]);
    let oauth = &v["components"]["securitySchemes"]["oauth2_auth"];
    assert_eq!(oauth["type"], "oauth2");
    // 2.0 `accessCode` flow → 3.0 `authorizationCode`
    let flow = &oauth["flows"]["authorizationCode"];
    assert_eq!(
        flow["authorizationUrl"],
        "https://example.com/oauth/authorize"
    );
    assert_eq!(flow["tokenUrl"], "https://example.com/oauth/token");
    assert_eq!(flow["scopes"]["read"], "read access");
}

#[test]
fn convert_reshapes_basic_security_scheme() {
    let v = run_json(&["convert", "3.0", SWAGGER2_YAML]);
    let basic = &v["components"]["securitySchemes"]["basic_auth"];
    assert_eq!(basic["type"], "http");
    assert_eq!(basic["scheme"], "basic");
}

#[test]
fn convert_keeps_apikey_scheme_shape() {
    let v = run_json(&["convert", "3.0", SWAGGER2_YAML]);
    let key = &v["components"]["securitySchemes"]["api_key"];
    assert_eq!(key["type"], "apiKey");
    assert_eq!(key["in"], "header");
    assert_eq!(key["name"], "X-API-Key");
}

#[test]
fn convert_rewrites_refs() {
    let value = run_json(&["convert", "3.0", "tests/fixtures/swagger2-with-ref.yaml"]);
    let schema_ref = &value["paths"]["/a"]["get"]["responses"]["200"]["content"]["application/json"]
        ["schema"]["$ref"];
    assert_eq!(schema_ref, "#/components/schemas/Thing");
    assert!(value["components"]["schemas"]["Thing"].is_object());
}

#[test]
fn convert_openapi30_to_openapi31_bumps_version() {
    let v = run_json(&[
        "convert",
        "3.1",
        "tests/fixtures/oai/petstore-expanded.yaml",
    ]);
    assert_eq!(v["openapi"], "3.1.0");
}

#[test]
fn convert_openapi30_to_openapi31_rewrites_nullable() {
    let v = run_json(&["convert", "3.1", "tests/fixtures/openapi30-nullable.yaml"]);
    assert_eq!(v["openapi"], "3.1.0");
    let name = &v["components"]["schemas"]["Item"]["properties"]["name"];
    assert!(name.get("nullable").is_none());
    assert_eq!(name["type"], json!(["string", "null"]));
}

#[test]
fn convert_openapi30_to_openapi31_rewrites_exclusive_bounds() {
    let v = run_json(&["convert", "3.1", "tests/fixtures/openapi30-nullable.yaml"]);
    let age = &v["components"]["schemas"]["Item"]["properties"]["age"];
    // exclusiveMinimum: true + minimum: 0  →  exclusiveMinimum: 0, no minimum
    assert_eq!(age["exclusiveMinimum"], 0);
    assert!(age.get("minimum").is_none());
    // exclusiveMaximum: false  →  drop exclusiveMaximum, keep maximum
    assert!(age.get("exclusiveMaximum").is_none());
    assert_eq!(age["maximum"], 120);
}

#[test]
fn convert_swagger2_to_openapi31_chains() {
    let v = run_json(&["convert", "3.1", SWAGGER2_YAML]);
    assert_eq!(v["openapi"], "3.1.0");
}

#[test]
fn convert_rejects_unsupported_direction() {
    let out = bin()
        .args(["convert", "2.0", PETSTORE_YAML])
        .output()
        .expect("spawn");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not implemented"));
}

#[test]
fn validate_accepts_valid_openapi() {
    let v = run_json(&["validate", PETSTORE_YAML]);
    assert_eq!(v["valid"], true);
    assert_eq!(v["version"], "3.1.0");
}

#[test]
fn validate_reports_swagger2_as_unsupported() {
    let v = run_json(&["validate", SWAGGER2_YAML]);
    assert!(v["valid"].is_null());
    assert_eq!(v["version"], "2.0");
    assert!(
        v["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("Swagger 2.0")
    );
}

#[test]
fn validate_reports_structural_error_with_pointer() {
    let out = bin()
        .args(["validate", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");
    use std::io::Write;
    out.stdin
        .as_ref()
        .unwrap()
        .write_all(b"openapi: \"3.1.0\"\ninfo: 42\npaths: {}\n")
        .unwrap();
    let result = out.wait_with_output().unwrap();
    assert!(result.status.success());
    let v: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(v["valid"], false);
    assert_eq!(v["errors"][0]["pointer"], "/info");
    assert!(
        v["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("expected")
    );
}

#[test]
fn tags_lists_declared_with_operation_count() {
    let tags = run_json(&["tags", PETSTORE_YAML]);
    assert_eq!(
        tags,
        json!([
            { "name": "pets", "description": "Everything about pets", "operationCount": 3 }
        ])
    );
}

#[test]
fn components_shows_only_populated_sections_by_default() {
    let components = run_json(&["components", PETSTORE_YAML]);
    assert_eq!(components["schemas"], json!(["Pet", "Pets"]));
    assert!(components.get("responses").is_none());
    assert!(components.get("parameters").is_none());
}

#[test]
fn components_show_null_emits_every_section() {
    let components = run_json(&["components", PETSTORE_YAML, "--show-null"]);
    for section in [
        "schemas",
        "responses",
        "parameters",
        "examples",
        "requestBodies",
        "headers",
        "securitySchemes",
        "links",
        "callbacks",
        "pathItems",
    ] {
        assert!(components[section].is_array(), "missing section: {section}");
    }
}

#[test]
fn max_depth_without_ref_resolution_warns() {
    let out = bin()
        .args(["operations", PETSTORE_YAML, "--max-depth", "3"])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--max-depth has no effect"));
}

#[test]
fn max_depth_silent_when_include_pulls_refs() {
    let out = bin()
        .args([
            "operations",
            PETSTORE_YAML,
            "--max-depth",
            "3",
            "--include",
            "response",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!stderr.contains("--max-depth"));
}

#[test]
fn no_resolve_refs_on_ref_free_command_warns() {
    let out = bin()
        .args(["info", PETSTORE_YAML, "--no-resolve-refs"])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--no-resolve-refs has no effect"));
}

#[test]
fn show_null_on_unsupported_command_warns() {
    let out = bin()
        .args(["paths", PETSTORE_YAML, "--show-null"])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--show-null has no effect"));
}

#[test]
fn schemas_lists_names() {
    let schemas = run_json(&["schemas", PETSTORE_YAML]);
    assert_eq!(schemas, json!(["Pet", "Pets"]));
}

#[test]
fn schema_resolves_refs_by_default() {
    let schema = run_json(&["schema", "Pets", PETSTORE_YAML]);
    assert_eq!(schema["type"], "array");
    // items should be the inlined Pet definition, not a $ref.
    assert!(schema["items"].get("$ref").is_none());
    assert_eq!(schema["items"]["type"], "object");
    assert_eq!(schema["items"]["properties"]["id"]["type"], "integer");
}

#[test]
fn schema_preserves_refs_with_flag() {
    let schema = run_json(&["schema", "Pets", PETSTORE_YAML, "--no-resolve-refs"]);
    assert_eq!(schema["items"]["$ref"], "#/components/schemas/Pet");
}

#[test]
fn schema_marks_circular_ref() {
    let schema = run_json(&["schema", "Node", CIRCULAR_YAML]);
    let marker = &schema["properties"]["children"]["items"]["$circular_ref"];
    assert_eq!(marker, "#/components/schemas/Node");
}

#[test]
fn overview_places_spec_version_and_servers_at_top_level() {
    let overview = run_json(&["overview", PETSTORE_YAML]);
    assert_eq!(overview["openapi"], "3.1.0");
    assert_eq!(overview["info"], run_json(&["info", PETSTORE_YAML]));
    assert_eq!(
        overview["servers"][0]["url"],
        "https://petstore.example.com/v1"
    );
    assert_eq!(overview["stats"], run_json(&["stats", PETSTORE_YAML]));
    assert_eq!(
        overview["operations"],
        run_json(&["operations", PETSTORE_YAML])
    );
}

#[test]
fn overview_uses_swagger_key_for_v2() {
    let overview = run_json(&["overview", SWAGGER2_YAML]);
    assert_eq!(overview["swagger"], "2.0");
    assert!(overview.get("openapi").is_none());
}

#[test]
fn missing_schema_errors() {
    let out = bin()
        .args(["schema", "Nope", PETSTORE_YAML])
        .output()
        .expect("spawn");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Nope"),
        "stderr should mention missing schema: {stderr}"
    );
}
