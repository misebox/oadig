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
