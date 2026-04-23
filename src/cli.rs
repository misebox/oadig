use clap::{Parser, Subcommand, ValueEnum};

/// Short doc shown as the description for the positional `<FILE>`.
const FILE_DOC: &str = "OpenAPI spec to read. `-` reads from stdin.";

#[derive(Debug, Parser)]
#[command(
    name = "oadig",
    version,
    about = "Extract specific info from OpenAPI specs"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, value_enum, default_value_t = Format::Json, global = true)]
    pub format: Format,

    /// Compact JSON output (no-op for YAML). JSON is pretty by default.
    #[arg(short = 'c', long, global = true, conflicts_with = "lines")]
    pub compact: bool,

    /// JSON: top-level array on multiple lines, each element on one line.
    /// Falls back to pretty for non-array values. No-op for YAML.
    #[arg(short = 'l', long, global = true)]
    pub lines: bool,

    /// Resolve $ref inline (default). Use --no-resolve-refs to disable.
    #[arg(
        long = "resolve-refs",
        default_value_t = true,
        global = true,
        overrides_with = "no_resolve_refs"
    )]
    pub resolve_refs: bool,

    #[arg(long = "no-resolve-refs", global = true)]
    pub no_resolve_refs: bool,

    #[arg(long, global = true)]
    pub max_depth: Option<usize>,

    /// Emit `null` for expected-but-absent fields instead of omitting the key.
    #[arg(long, global = true)]
    pub show_null: bool,
}

impl Cli {
    pub fn should_resolve_refs(&self) -> bool {
        !self.no_resolve_refs
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[value(rename_all = "camelCase")]
pub enum StatusField {
    Headers,
    Schema,
    /// Expands to every other field.
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[value(rename_all = "camelCase")]
pub enum OperationField {
    Summary,
    Description,
    Tags,
    Parameters,
    Request,
    Response,
    Security,
    Deprecated,
    OperationId,
    /// Expands to every other field.
    All,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show title, version, description, contact, license, servers.
    Info {
        #[arg(help = FILE_DOC)]
        file: String,
    },
    /// Emit the spec version string (openapi 3.x or swagger 2.0).
    Spec {
        #[arg(help = FILE_DOC)]
        file: String,
    },
    /// Show counts: paths, operations, schemas, tags, methods.
    Stats {
        #[arg(help = FILE_DOC)]
        file: String,
    },
    /// Combined info + stats + operations.
    Overview {
        #[arg(help = FILE_DOC)]
        file: String,
    },
    /// List path strings (keys of the `paths` object).
    Paths {
        #[arg(help = FILE_DOC)]
        file: String,
        /// Filter as key=value. Only `path=` is supported here.
        ///
        /// `*` acts as a wildcard at the start and/or end of the value
        /// (quote to protect from the shell): `*foo*` contains, `foo*`
        /// prefix, `*foo` suffix, `foo` exact.
        #[arg(long = "filter")]
        filters: Vec<String>,
    },
    /// List operations (method + path, with configurable extras).
    #[command(alias = "ops")]
    Operations {
        #[arg(help = FILE_DOC)]
        file: String,
        /// Filter as key=value. Repeat for AND.
        ///
        /// Keys: method, path, tag, operationId, summary, description, deprecated.
        ///
        /// `method` and `tag` take one value or a comma list (OR within).
        /// Other keys use `*` as a wildcard at start/end of value (quote
        /// to protect from the shell): `*foo*` contains, `foo*` prefix,
        /// `*foo` suffix, `foo` exact. `deprecated` takes `true`/`false`.
        #[arg(long = "filter")]
        filters: Vec<String>,
        /// Extra fields per entry.
        ///
        /// Default: summary.
        ///
        /// Values: summary, description, tags, parameters, request, response, security, deprecated, operationId, all.
        #[arg(long, value_enum, value_delimiter = ',', hide_possible_values = true)]
        include: Vec<OperationField>,
        /// Fields to remove per entry.
        ///
        /// Same values as --include.
        #[arg(long, value_enum, value_delimiter = ',', hide_possible_values = true)]
        exclude: Vec<OperationField>,
    },
    /// Show a single operation with every field, $refs resolved.
    ///
    /// Two call shapes:
    ///   oadig operation <ID> <FILE>            lookup by operationId
    ///   oadig operation <FILE> -m GET -p /x    lookup by method + path
    #[command(alias = "op")]
    Operation {
        /// Either `<ID> <FILE>` (two args) or `<FILE>` with -m/-p.
        #[arg(num_args = 1..=2)]
        args: Vec<String>,
        /// HTTP method (use with -p). Required when no operationId is given.
        #[arg(short = 'm', long, requires = "path")]
        method: Option<String>,
        /// Path template (use with -m).
        #[arg(short = 'p', long, requires = "method")]
        path: Option<String>,
    },
    /// List requestBodies of operations that have one.
    Requests {
        #[arg(help = FILE_DOC)]
        file: String,
    },
    /// Show the requestBody of a single operation, $refs resolved.
    ///
    /// Same call shapes as `operation`.
    #[command(alias = "req")]
    Request {
        #[arg(num_args = 1..=2)]
        args: Vec<String>,
        #[arg(short = 'm', long, requires = "path")]
        method: Option<String>,
        #[arg(short = 'p', long, requires = "method")]
        path: Option<String>,
    },
    /// List unique status codes used across the spec with a description.
    Statuses {
        #[arg(help = FILE_DOC)]
        file: String,
        /// Extra fields per entry.
        ///
        /// Default: status, description.
        ///
        /// Values: headers, schema, all.
        #[arg(long, value_enum, value_delimiter = ',', hide_possible_values = true)]
        include: Vec<StatusField>,
    },
    /// List responses of every operation. Optionally narrow to one status.
    Responses {
        #[arg(help = FILE_DOC)]
        file: String,
        /// Narrow each entry to a single status code (e.g. 200).
        #[arg(long)]
        status: Option<String>,
    },
    /// Show the responses of a single operation, $refs resolved.
    ///
    /// Same call shapes as `operation`.
    #[command(alias = "res")]
    Response {
        #[arg(num_args = 1..=2)]
        args: Vec<String>,
        #[arg(short = 'm', long, requires = "path")]
        method: Option<String>,
        #[arg(short = 'p', long, requires = "method")]
        path: Option<String>,
        /// Narrow to a single status code (e.g. 200).
        #[arg(long)]
        status: Option<String>,
    },
    /// Search string values in the spec for a keyword. Returns JSON Pointers.
    Search {
        /// Keyword or regex to match.
        keyword: String,
        #[arg(help = FILE_DOC)]
        file: String,
        /// Treat keyword as a regex instead of a substring.
        #[arg(long)]
        regex: bool,
        /// Case-sensitive match (default: case-insensitive).
        #[arg(long)]
        case_sensitive: bool,
    },
    /// List declared and referenced tags with operation counts.
    Tags {
        #[arg(help = FILE_DOC)]
        file: String,
    },
    /// Show component sections and the names defined in each.
    Components {
        #[arg(help = FILE_DOC)]
        file: String,
    },
    /// List component schema names.
    Schemas {
        #[arg(help = FILE_DOC)]
        file: String,
    },
    /// Show a single component schema definition.
    Schema {
        /// Schema name from components.schemas.
        name: String,
        #[arg(help = FILE_DOC)]
        file: String,
    },
}
