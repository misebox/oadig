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
fn endpoints_alias_resolves_to_operations() {
    let from_alias = run_json(&["endpoints", PETSTORE_YAML]);
    let from_canonical = run_json(&["operations", PETSTORE_YAML]);
    assert_eq!(from_alias, from_canonical);
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
