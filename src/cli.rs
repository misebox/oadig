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
        /// Keep only paths that contain this substring.
        #[arg(long)]
        filter: Option<String>,
        /// Keep only paths that start with this prefix.
        #[arg(long)]
        prefix: Option<String>,
    },
    /// List operations (method + path, with configurable extras).
    #[command(alias = "ops")]
    Operations {
        #[arg(help = FILE_DOC)]
        file: String,
        /// Keep only operations with these HTTP methods (comma-separated).
        #[arg(short = 'm', long, value_delimiter = ',')]
        method: Vec<String>,
        /// Keep only operations whose path contains this substring.
        #[arg(long)]
        filter: Option<String>,
        /// Keep only operations whose path starts with this prefix.
        #[arg(long)]
        prefix: Option<String>,
        /// Keep only operations tagged with this name.
        #[arg(long)]
        tag: Option<String>,
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
    /// Identify the operation by its operationId (positional) OR by
    /// -m/--method and -p/--path. The two forms are mutually exclusive.
    #[command(alias = "op")]
    Operation {
        #[arg(help = FILE_DOC)]
        file: String,
        /// operationId to look up.
        id: Option<String>,
        /// HTTP method (use with -p).
        #[arg(short = 'm', long, conflicts_with = "id", requires = "path")]
        method: Option<String>,
        /// Path template (use with -m).
        #[arg(short = 'p', long, conflicts_with = "id", requires = "method")]
        path: Option<String>,
    },
    /// List requestBodies of operations that have one.
    Requests {
        #[arg(help = FILE_DOC)]
        file: String,
    },
    /// Show the requestBody of a single operation, $refs resolved.
    #[command(alias = "req")]
    Request {
        #[arg(help = FILE_DOC)]
        file: String,
        /// operationId to look up.
        id: Option<String>,
        #[arg(short = 'm', long, conflicts_with = "id", requires = "path")]
        method: Option<String>,
        #[arg(short = 'p', long, conflicts_with = "id", requires = "method")]
        path: Option<String>,
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
    #[command(alias = "res")]
    Response {
        #[arg(help = FILE_DOC)]
        file: String,
        /// operationId to look up.
        id: Option<String>,
        #[arg(short = 'm', long, conflicts_with = "id", requires = "path")]
        method: Option<String>,
        #[arg(short = 'p', long, conflicts_with = "id", requires = "method")]
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
