use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "oadig", version, about = "Extract specific info from OpenAPI specs")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, value_enum, default_value_t = Format::Json, global = true)]
    pub format: Format,

    /// Compact JSON output (no-op for YAML). JSON is pretty by default.
    #[arg(short = 'c', long, global = true)]
    pub compact: bool,

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

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show title, version, description, contact, license, servers.
    Info {
        /// Path to an OpenAPI spec (`-` for stdin).
        file: String,
    },
    /// Show counts: paths, operations, schemas, tags, methods.
    Stats {
        file: String,
    },
    /// List method + path pairs.
    Paths {
        file: String,
    },
    /// List component schema names.
    Schemas {
        file: String,
    },
    /// Show a single component schema definition.
    Schema {
        name: String,
        file: String,
    },
}
