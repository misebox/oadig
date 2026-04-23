use clap::{Parser, Subcommand, ValueEnum};

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
        /// Path to an OpenAPI spec (`-` for stdin).
        file: String,
    },
    /// Combined info + stats + paths.
    Overview { file: String },
    /// Show counts: paths, operations, schemas, tags, methods.
    Stats { file: String },
    /// List path strings (keys of the `paths` object).
    Paths { file: String },
    /// List operations (method + path, with configurable extras).
    #[command(alias = "endpoints")]
    Operations {
        file: String,
        /// Extra fields to include in each entry. Default: summary.
        #[arg(long, value_enum, value_delimiter = ',')]
        include: Vec<OperationField>,
        /// Fields to remove from each entry.
        #[arg(long, value_enum, value_delimiter = ',')]
        exclude: Vec<OperationField>,
    },
    /// List component schema names.
    Schemas { file: String },
    /// Show a single component schema definition.
    Schema { name: String, file: String },
}
