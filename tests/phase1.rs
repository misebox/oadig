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
fn info_show_null_fills_missing_keys() {
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
    assert!(value["openapi"].is_null());
    assert!(value["description"].is_null());
    assert!(value["contact"].is_null());
    assert!(value["license"].is_null());
    assert!(value["servers"].is_null());
}

#[test]
fn info_surfaces_swagger_version_for_v2_specs() {
    let info = run_json(&["info", SWAGGER2_YAML]);
    assert_eq!(info["swagger"], "2.0");
    assert!(info.get("openapi").is_none());
    assert_eq!(info["title"], "Legacy API");
}

#[test]
fn info_omits_spec_version_key_when_missing() {
    // Feed a minimal spec via stdin that has neither openapi nor swagger.
    let out = bin()
        .args(["info", "-"])
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
    assert!(value.get("openapi").is_none());
    assert!(value.get("swagger").is_none());
    assert_eq!(value["title"], "Orphan");
}

#[test]
fn info_yaml_and_json_match() {
    let from_yaml = run_json(&["info", PETSTORE_YAML]);
    let from_json = run_json(&["info", PETSTORE_JSON]);
    assert_eq!(from_yaml, from_json);
    assert_eq!(from_yaml["title"], "Petstore");
    assert_eq!(from_yaml["version"], "1.0.0");
    assert_eq!(
        from_yaml["servers"][0]["url"],
        "https://petstore.example.com/v1"
    );
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
fn paths_filter_keeps_only_matches_by_substring() {
    let paths = run_json(&["paths", PETSTORE_YAML, "--filter", "petId"]);
    assert_eq!(paths, json!(["/pets/{petId}"]));
}

#[test]
fn paths_filter_by_prefix() {
    let paths = run_json(&["paths", PETSTORE_YAML, "--prefix", "/pets/"]);
    assert_eq!(paths, json!(["/pets/{petId}"]));
}

#[test]
fn operations_filter_by_method() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--method", "POST"]);
    let arr = ops.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["method"], "POST");
}

#[test]
fn operations_filter_by_multiple_methods() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--method", "GET,POST"]);
    assert_eq!(ops.as_array().unwrap().len(), 3);
}

#[test]
fn operations_filter_by_path_substring() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--filter", "petId"]);
    let arr = ops.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["path"], "/pets/{petId}");
}

#[test]
fn operations_filter_by_prefix() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--prefix", "/pets/"]);
    let arr = ops.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["path"], "/pets/{petId}");
}

#[test]
fn operations_filter_combines_substring_and_prefix_as_and() {
    let ops = run_json(&[
        "operations",
        PETSTORE_YAML,
        "--prefix",
        "/pets",
        "--filter",
        "petId",
    ]);
    let arr = ops.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["path"], "/pets/{petId}");
}

#[test]
fn operations_filter_by_tag() {
    let ops = run_json(&["operations", PETSTORE_YAML, "--tag", "pets"]);
    assert_eq!(ops.as_array().unwrap().len(), 3);
    let none = run_json(&["operations", PETSTORE_YAML, "--tag", "nonexistent"]);
    assert!(none.as_array().unwrap().is_empty());
}

#[test]
fn operations_lists_method_path_summary() {
    let ops = run_json(&["operations", PETSTORE_YAML]);
    let expected = json!([
        { "method": "GET",  "path": "/pets",         "summary": "List all pets" },
        { "method": "POST", "path": "/pets",         "summary": "Create a pet" },
        { "method": "GET",  "path": "/pets/{petId}", "summary": "Info for a specific pet" },
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
    let schema = &get_pets["response"]["200"]["content"]["application/json"]["schema"];
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
    let op = run_json(&["operation", PETSTORE_YAML, "listPets"]);
    assert_eq!(op["method"], "GET");
    assert_eq!(op["path"], "/pets");
    assert_eq!(op["operationId"], "listPets");
    let schema = &op["responses"]["200"]["content"]["application/json"]["schema"];
    assert!(schema.get("$ref").is_none());
    assert_eq!(schema["type"], "array");
}

#[test]
fn operation_lookup_by_method_and_path_matches_id() {
    let by_id = run_json(&["operation", PETSTORE_YAML, "listPets"]);
    let by_mp = run_json(&["operation", PETSTORE_YAML, "-m", "GET", "-p", "/pets"]);
    assert_eq!(by_id, by_mp);
}

#[test]
fn op_alias_resolves_to_operation() {
    let from_alias = run_json(&["op", PETSTORE_YAML, "listPets"]);
    let from_canonical = run_json(&["operation", PETSTORE_YAML, "listPets"]);
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
        .args(["operation", PETSTORE_YAML, "nonexistent"])
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
fn operation_rejects_id_mixed_with_flags() {
    let out = bin()
        .args([
            "operation",
            PETSTORE_YAML,
            "listPets",
            "-m",
            "GET",
            "-p",
            "/pets",
        ])
        .output()
        .expect("spawn");
    assert!(!out.status.success());
}

#[test]
fn response_narrows_by_status() {
    let all = run_json(&["response", PETSTORE_YAML, "listPets"]);
    assert!(all.get("200").is_some());
    let only_200 = run_json(&["response", PETSTORE_YAML, "listPets", "--status", "200"]);
    assert_eq!(only_200, all["200"]);
}

#[test]
fn res_alias_resolves_to_response() {
    let from_alias = run_json(&["res", PETSTORE_YAML, "listPets"]);
    let from_canonical = run_json(&["response", PETSTORE_YAML, "listPets"]);
    assert_eq!(from_alias, from_canonical);
}

#[test]
fn request_returns_null_when_missing() {
    // petstore listPets has no requestBody
    let req = run_json(&["request", PETSTORE_YAML, "listPets"]);
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
    let schema = &arr[0]["request"]["content"]["application/json"]["schema"];
    assert!(schema.get("$ref").is_none());
    assert_eq!(schema["type"], "object");
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
fn overview_combines_info_stats_operations() {
    let overview = run_json(&["overview", PETSTORE_YAML]);
    assert_eq!(overview["info"], run_json(&["info", PETSTORE_YAML]));
    assert_eq!(overview["stats"], run_json(&["stats", PETSTORE_YAML]));
    assert_eq!(
        overview["operations"],
        run_json(&["operations", PETSTORE_YAML])
    );
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
