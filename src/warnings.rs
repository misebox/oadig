use crate::cli::{Cli, Command, OperationField, StatusField};

/// Return a list of warnings for flags that were explicitly set but have
/// no effect on the chosen subcommand with the given arguments.
pub fn for_invocation(args: &Cli) -> Vec<String> {
    let mut out = Vec::new();
    let cmd_name = subcommand_name(&args.command);

    if args.max_depth.is_some() && !command_resolves_refs(&args.command) {
        out.push(format!(
            "warning: --max-depth has no effect on `{cmd_name}` with these flags (it only applies when $refs are resolved)"
        ));
    }
    if args.no_resolve_refs && !command_can_touch_refs(&args.command) {
        out.push(format!(
            "warning: --no-resolve-refs has no effect on `{cmd_name}` (it does not resolve $refs)"
        ));
    }
    if args.show_null && !command_uses_show_null(&args.command) {
        out.push(format!(
            "warning: --show-null has no effect on `{cmd_name}`"
        ));
    }

    out
}

fn subcommand_name(cmd: &Command) -> &'static str {
    match cmd {
        Command::Info { .. } => "info",
        Command::Spec { .. } => "spec",
        Command::Stat { .. } => "stat",
        Command::Overview { .. } => "overview",
        Command::Paths { .. } => "paths",
        Command::Operations { .. } => "operations",
        Command::Operation { .. } => "operation",
        Command::Requests { .. } => "requests",
        Command::Request { .. } => "request",
        Command::Responses { .. } => "responses",
        Command::Statuses { .. } => "statuses",
        Command::Response { .. } => "response",
        Command::Search { .. } => "search",
        Command::Tags { .. } => "tags",
        Command::Components { .. } => "components",
        Command::Schemas { .. } => "schemas",
        Command::Schema { .. } => "schema",
    }
}

// Whether this invocation will actually traverse $refs.
// `operations` resolves only when --include pulls in parameters/request/response.
fn command_resolves_refs(cmd: &Command) -> bool {
    match cmd {
        Command::Info { .. }
        | Command::Spec { .. }
        | Command::Stat { .. }
        | Command::Overview { .. }
        | Command::Paths { .. }
        | Command::Tags { .. }
        | Command::Components { .. }
        | Command::Schemas { .. }
        | Command::Search { .. } => false,
        Command::Statuses { include, .. } => include.iter().any(|f| {
            matches!(
                f,
                StatusField::Schema
                    | StatusField::Headers
                    | StatusField::Content
                    | StatusField::All
            )
        }),
        Command::Operations { include, .. } => include.iter().any(|f| {
            matches!(
                f,
                OperationField::Parameters
                    | OperationField::Request
                    | OperationField::Response
                    | OperationField::All
            )
        }),
        Command::Operation { .. }
        | Command::Request { .. }
        | Command::Response { .. }
        | Command::Requests { .. }
        | Command::Responses { .. }
        | Command::Schema { .. } => true,
    }
}

fn command_can_touch_refs(cmd: &Command) -> bool {
    // Same set as commands that could ever resolve, regardless of flags.
    match cmd {
        Command::Info { .. }
        | Command::Spec { .. }
        | Command::Stat { .. }
        | Command::Overview { .. }
        | Command::Paths { .. }
        | Command::Tags { .. }
        | Command::Components { .. }
        | Command::Schemas { .. }
        | Command::Search { .. } => false,
        _ => true,
    }
}

fn command_uses_show_null(cmd: &Command) -> bool {
    matches!(
        cmd,
        Command::Info { .. } | Command::Overview { .. } | Command::Components { .. }
    )
}
