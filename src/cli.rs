use clap::{Args as ClapArgs, Parser, Subcommand, ValueEnum};
use kicad_ipc_rs::{DocumentType, EditorFrameType};

const ROOT_AFTER_HELP: &str = "Examples:\n  ki refresh --frame pcb\n  ki extract board.kicad_sch --pretty\n  ki schematic inspect board.kicad_sch --json\n  ki pcb add-trace board.kicad_pcb 10 10 20 10 0.25 F.Cu 1\n  ki project validate project.kicad_pro --json";
const ROOT_USAGE_TEXT: &str = "ki - KiCad CLI for IPC refresh and file editing

USAGE:
  ki refresh [--socket <SOCKET>] [--token <TOKEN>] [--client-name <CLIENT_NAME>] [--timeout-ms <TIMEOUT_MS>] [--frame <FRAME>] [--silent]
  ki extract [--pretty] [--output <FILE>] [--include-nets] [--include-diagnostics] [--sym-lib <FILE>] <INPUT>
  ki project <action> [args] [flags]
  ki schematic <action> [args] [flags]
  ki symbol-lib <action> [args] [flags]
  ki pcb <action> [args] [flags]
  ki lib-table <action> [args] [flags]

COMMANDS:
  refresh
    Refresh a KiCad editor frame through IPC.

  extract
    Extract netlist/topology JSON from <path.kicad_sch>

  project
    open <path.kicad_pro>
    validate <path.kicad_pro>

  schematic
    inspect <path.kicad_sch>
    set-property <path.kicad_sch> <reference> <key> <value>
    remove-property <path.kicad_sch> <reference> <key>
    add-symbol <path.kicad_sch> <lib_id> <reference> <value> <x> <y>
    remove-symbol <path.kicad_sch> <reference>
    rename <path.kicad_sch> <reference> <new_lib_id>
    add-wire <path.kicad_sch> <x1> <y1> <x2> <y2>
    remove-wire <path.kicad_sch> <x1> <y1> <x2> <y2>
    add-label <path.kicad_sch> <text> <x> <y> <angle>
    add-global-label <path.kicad_sch> <text> <shape> <x> <y> <angle>
    remove-label <path.kicad_sch> <name>
    rename-label <path.kicad_sch> <name> <new_name>
    add-junction <path.kicad_sch> <x> <y>
    add-no-connect <path.kicad_sch> <x> <y>
    fork-symbol <path.kicad_sch> <reference> <library_name> <target_symbol_name> [--override]
    push-to-lib <path.kicad_sch> <reference> <library_name>
    replace-from-lib <path.kicad_sch> <reference> <library_name> <symbol_name> [--override-value] [--preserve-property <name>...]
    update-from-lib <path.kicad_sch> <library_name> <reference> [--override-value] [--preserve-property <name>...]
    update-from-lib <path.kicad_sch> <library_name> --all [--override-value] [--preserve-property <name>...]
    erc <path.kicad_sch>
    query component <path.kicad_sch> <reference>
    query net <path.kicad_sch> <net_name> [--hierarchical]
    query unconnected <path.kicad_sch> [--hierarchical]
    check-intent <path.kicad_sch> --intent <file.json>

  symbol-lib
    inspect <path.kicad_sym> [symbol]
    set-property <path.kicad_sym> <symbol> <key> <value>
    remove-property <path.kicad_sym> <symbol> <key>
    rename <path.kicad_sym> <from> <to>

  pcb
    inspect <path.kicad_pcb>
    query footprint <path.kicad_pcb> <reference>
    set-property <path.kicad_pcb> <key> <value>
    add-trace <path.kicad_pcb> <x1> <y1> <x2> <y2> <width> <layer> <net>
    remove-trace <path.kicad_pcb> <x1> <y1> <x2> <y2>
    add-via <path.kicad_pcb> <x> <y> <size> <drill> <net>
    add-footprint <path.kicad_pcb> <lib_ref> <x> <y> <layer> <reference> <value>
    move-footprint <path.kicad_pcb> <reference> <x> <y> [rotation]
    remove-footprint <path.kicad_pcb> <reference>

  lib-table
    inspect <path>
    add <path> <name> <uri>
    rename <path> <from> <to>

FLAGS:
  --json            Machine-readable JSON to stdout
  --diagnostics     Emit diagnostics as JSON array to stderr
  --hierarchical    Load all sub-sheets and merge nets via global labels

EXTRACT FLAGS:
  --pretty                    Pretty-print JSON output
  --output <FILE>             Write output JSON to FILE
  --include-nets              Include top-level net connectivity in output
  --include-diagnostics       Include extract diagnostics in output
  --sym-lib <FILE>            Enrich lib_parts from a .kicad_sym file (repeatable)

REFRESH FLAGS:
  --socket <SOCKET>            Override the KiCad IPC socket URI/path
  --token <TOKEN>              Override the KiCad instance token
  --client-name <CLIENT_NAME>  Set the client name sent to KiCad
  --timeout-ms <TIMEOUT_MS>    IPC request timeout in milliseconds
  --frame <FRAME>              Target editor frame
  --silent                     Suppress refresh output and return success on refresh errors

EXIT CODES:
  0   success
  1   validation warnings or errors found, or IPC refresh failure
  2   parse or IO error";

const SCHEMATIC_AFTER_LONG_HELP: &str = "Actions:\n  inspect <path.kicad_sch>\n  set-property <path.kicad_sch> <reference> <key> <value>\n  remove-property <path.kicad_sch> <reference> <key>\n  add-symbol <path.kicad_sch> <lib_id> <reference> <value> <x> <y>\n  remove-symbol <path.kicad_sch> <reference>\n  rename <path.kicad_sch> <reference> <new_lib_id>\n  add-wire <path.kicad_sch> <x1> <y1> <x2> <y2>\n  remove-wire <path.kicad_sch> <x1> <y1> <x2> <y2>\n  add-label <path.kicad_sch> <text> <x> <y> <angle>\n  add-global-label <path.kicad_sch> <text> <shape> <x> <y> <angle>\n  add-junction <path.kicad_sch> <x> <y>\n  add-no-connect <path.kicad_sch> <x> <y>\n  fork-symbol <path.kicad_sch> <reference> <library_name> <target_symbol_name> [--override]\n  push-to-lib <path.kicad_sch> <reference> <library_name>\n  replace-from-lib <path.kicad_sch> <reference> <library_name> <symbol_name> [--override-value] [--preserve-property <name>...]\n  update-from-lib <path.kicad_sch> <library_name> <reference> [--override-value] [--preserve-property <name>...]\n  update-from-lib <path.kicad_sch> <library_name> --all [--override-value] [--preserve-property <name>...]\n  erc <path.kicad_sch> [--json] [--output <FILE>] [--units <UNITS>] [--severity-all|--severity-error|--severity-warning|--severity-exclusions] [--exit-code-violations]\n  query component <path.kicad_sch> <reference>\n  query net <path.kicad_sch> <net_name> [--hierarchical]\n  query unconnected <path.kicad_sch> [--hierarchical]\n  check-intent <path.kicad_sch> --intent <file.json>\n\nFlags:\n  --json            Machine-readable JSON to stdout\n  --diagnostics     Emit diagnostics as JSON array to stderr";

const PCB_AFTER_LONG_HELP: &str = "Actions:\n  inspect <path.kicad_pcb>\n  query footprint <path.kicad_pcb> <reference>\n  set-property <path.kicad_pcb> <key> <value>\n  add-trace <path.kicad_pcb> <x1> <y1> <x2> <y2> <width> <layer> <net>\n  remove-trace <path.kicad_pcb> <x1> <y1> <x2> <y2>\n  add-via <path.kicad_pcb> <x> <y> <size> <drill> <net>\n  add-footprint <path.kicad_pcb> <lib_ref> <x> <y> <layer> <reference> <value>\n  move-footprint <path.kicad_pcb> <reference> <x> <y> [rotation]\n  remove-footprint <path.kicad_pcb> <reference>\n\nFlags:\n  --json            Machine-readable JSON to stdout\n  --diagnostics     Emit diagnostics as JSON array to stderr";

const PROJECT_AFTER_LONG_HELP: &str = "Actions:\n  open <path.kicad_pro>\n  validate <path.kicad_pro>\n\nFlags:\n  --json            Machine-readable JSON to stdout\n  --diagnostics     Emit diagnostics as JSON array to stderr";

const SYMBOL_LIB_AFTER_LONG_HELP: &str = "Actions:\n  inspect <path.kicad_sym> [symbol]\n  set-property <path.kicad_sym> <symbol> <key> <value>\n  remove-property <path.kicad_sym> <symbol> <key>\n  rename <path.kicad_sym> <from> <to>\n\nFlags:\n  --json            Machine-readable JSON to stdout\n  --diagnostics     Emit diagnostics as JSON array to stderr";

const LIB_TABLE_AFTER_LONG_HELP: &str = "Actions:\n  inspect <path>\n  add <path> <name> <uri>\n  rename <path> <from> <to>\n\nFlags:\n  --json            Machine-readable JSON to stdout\n  --diagnostics     Emit diagnostics as JSON array to stderr";
const EXTRACT_AFTER_LONG_HELP: &str = "Arguments:\n  <INPUT>                      .kicad_sch schematic file\n\nFlags:\n  -p, --pretty                 Pretty-print JSON output\n  -o, --output <FILE>          Write output JSON to FILE (default: stdout)\n      --include-nets           Include top-level net connectivity in output\n      --include-diagnostics    Include extract diagnostics in output\n      --sym-lib <FILE>         Path to a .kicad_sym file for pin/lib-part enrichment (repeatable)\n      --verbose                Emit verbose enrichment logs to stderr";

#[derive(Debug, Parser)]
#[command(
    name = "ki",
    about = "KiCad CLI for IPC refresh and file editing",
    after_help = ROOT_AFTER_HELP
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

pub fn print_root_usage() {
    println!("{ROOT_USAGE_TEXT}");
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Refresh a KiCad editor frame.
    Refresh(RefreshArgs),
    /// Extract netlist/topology JSON from a schematic.
    Extract(ExtractArgs),
    /// Open or validate a KiCad project.
    Project(ProjectCommand),
    /// Inspect or edit a schematic.
    Schematic(SchematicCommand),
    /// Inspect or edit a symbol library.
    #[command(name = "symbol-lib")]
    SymbolLib(SymbolLibCommand),
    /// Inspect or edit a PCB.
    Pcb(PcbCommand),
    /// Inspect or edit a library table.
    #[command(name = "lib-table")]
    LibTable(LibTableCommand),
}

#[derive(Debug, Clone, ClapArgs)]
#[command(
    about = "Extract netlist/topology JSON from a schematic",
    after_long_help = EXTRACT_AFTER_LONG_HELP
)]
pub struct ExtractArgs {
    /// .kicad_sch schematic file.
    pub input: String,

    /// Pretty-print JSON output.
    #[arg(short, long)]
    pub pretty: bool,

    /// Write output JSON to FILE (default: stdout).
    #[arg(short, long)]
    pub output: Option<String>,

    /// Include top-level net connectivity in output.
    #[arg(long)]
    pub include_nets: bool,

    /// Include extract diagnostics in output.
    #[arg(long)]
    pub include_diagnostics: bool,

    /// Path to a .kicad_sym file for pin/lib-part enrichment (repeatable).
    #[arg(long = "sym-lib")]
    pub sym_lib: Vec<String>,

    /// Emit verbose enrichment logs to stderr.
    #[arg(long)]
    pub verbose: bool,
}

#[derive(Debug, clap::Args)]
pub struct RefreshArgs {
    /// Override the KiCad IPC socket URI/path.
    #[arg(long)]
    pub socket: Option<String>,

    /// Override the KiCad instance token.
    #[arg(long)]
    pub token: Option<String>,

    /// Set the client name sent to KiCad.
    #[arg(long)]
    pub client_name: Option<String>,

    /// IPC request timeout in milliseconds.
    #[arg(long, default_value_t = 3_000)]
    pub timeout_ms: u64,

    /// Frame/editor to refresh.
    #[arg(long, value_enum, default_value_t = FrameArg::Schematic)]
    pub frame: FrameArg,

    /// Suppress all refresh output while preserving exit status.
    #[arg(long)]
    pub silent: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum FrameArg {
    #[value(name = "schematic")]
    Schematic,
    #[value(name = "pcb")]
    Pcb,
    #[value(name = "project-manager")]
    ProjectManager,
    #[value(name = "spice")]
    Spice,
    #[value(name = "symbol")]
    Symbol,
    #[value(name = "footprint")]
    Footprint,
    #[value(name = "drawing-sheet")]
    DrawingSheet,
}

impl FrameArg {
    pub fn into_editor_frame(self) -> EditorFrameType {
        match self {
            Self::Schematic => EditorFrameType::SchematicEditor,
            Self::Pcb => EditorFrameType::PcbEditor,
            Self::ProjectManager => EditorFrameType::ProjectManager,
            Self::Spice => EditorFrameType::SpiceSimulator,
            Self::Symbol => EditorFrameType::SymbolEditor,
            Self::Footprint => EditorFrameType::FootprintEditor,
            Self::DrawingSheet => EditorFrameType::DrawingSheetEditor,
        }
    }

    pub fn into_document_type(self) -> Option<DocumentType> {
        match self {
            Self::Schematic => Some(DocumentType::Schematic),
            Self::Pcb => Some(DocumentType::Pcb),
            Self::ProjectManager => Some(DocumentType::Project),
            Self::Spice => None,
            Self::Symbol => Some(DocumentType::Symbol),
            Self::Footprint => Some(DocumentType::Footprint),
            Self::DrawingSheet => Some(DocumentType::DrawingSheet),
        }
    }
}

#[derive(Debug, Clone, ClapArgs)]
pub struct OutputArgs {
    #[arg(long)]
    pub json: bool,

    #[arg(long)]
    pub diagnostics: bool,
}

#[derive(Debug, Subcommand)]
pub enum ProjectAction {
    /// Open a KiCad project and summarize its contents.
    Open {
        path: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Validate a KiCad project against its library tables.
    Validate {
        path: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Debug, ClapArgs)]
#[command(about = "Open or validate a KiCad project", after_long_help = PROJECT_AFTER_LONG_HELP)]
pub struct ProjectCommand {
    #[command(subcommand)]
    pub action: ProjectAction,
}

#[derive(Debug, Subcommand)]
pub enum SchematicAction {
    /// Inspect a schematic.
    Inspect {
        path: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Set a component property in a schematic.
    SetProperty {
        path: String,
        reference: String,
        key: String,
        value: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Remove a component property in a schematic.
    RemoveProperty {
        path: String,
        reference: String,
        key: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a symbol instance to a schematic.
    AddSymbol {
        path: String,
        lib_id: String,
        reference: String,
        value: String,
        x: f64,
        y: f64,
        #[command(flatten)]
        output: OutputArgs,
    },

    /// Remove a symbol instance from a schematic.
    RemoveSymbol {
        path: String,
        reference: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    #[command(alias = "set-lib-id")]
    /// Change the library symbol used by a schematic reference.
    Rename {
        path: String,
        reference: String,
        new_lib_id: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a wire segment.
    AddWire {
        path: String,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Remove a wire segment.
    RemoveWire {
        path: String,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a local label.
    AddLabel {
        path: String,
        text: String,
        x: f64,
        y: f64,
        angle: f64,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a global label.
    AddGlobalLabel {
        path: String,
        text: String,
        shape: String,
        x: f64,
        y: f64,
        angle: f64,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Remove all labels with the given text.
    RemoveLabel {
        path: String,
        name: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Rename all labels with the given text to a new name.
    RenameLabel {
        path: String,
        name: String,
        new_name: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a junction marker.
    AddJunction {
        path: String,
        x: f64,
        y: f64,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a no-connect marker.
    AddNoConnect {
        path: String,
        x: f64,
        y: f64,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Fork a schematic symbol into a project library.
    ForkSymbol {
        path: String,
        reference: String,
        library_name: String,
        target_symbol_name: String,
        #[arg(long = "override")]
        overwrite: bool,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Push a schematic symbol into a project library.
    PushToLib {
        path: String,
        reference: String,
        library_name: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Replace a schematic symbol from a library symbol.
    ReplaceFromLib {
        path: String,
        reference: String,
        library_name: String,
        symbol_name: String,
        #[arg(long)]
        override_value: bool,
        #[arg(long = "preserve-property")]
        preserve_property: Vec<String>,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Update one or more embedded schematic symbols from a library.
    UpdateFromLib {
        path: String,
        library_name: String,
        reference: Option<String>,
        #[arg(long, conflicts_with = "reference")]
        all: bool,
        #[arg(long)]
        override_value: bool,
        #[arg(long = "preserve-property")]
        preserve_property: Vec<String>,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Run electrical rules checks on a schematic.
    Erc {
        path: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long, default_value = "mm")]
        units: String,
        #[arg(long)]
        severity_all: bool,
        #[arg(long)]
        severity_error: bool,
        #[arg(long)]
        severity_warning: bool,
        #[arg(long)]
        severity_exclusions: bool,
        #[arg(long)]
        exit_code_violations: bool,
        #[command(flatten)]
        format: OutputArgs,
    },
    /// Query schematic components or nets.
    Query(SchematicQueryCommand),
    /// Check a schematic against a JSON intent file.
    CheckIntent {
        path: String,
        #[arg(long)]
        intent: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Debug, ClapArgs)]
#[command(
    about = "Inspect or edit a schematic",
    after_long_help = SCHEMATIC_AFTER_LONG_HELP
)]
pub struct SchematicCommand {
    #[command(subcommand)]
    pub action: SchematicAction,
}

#[derive(Debug, Subcommand)]
pub enum SchematicQueryAction {
    /// Query a component by reference.
    Component {
        path: String,
        reference: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Query a named net.
    Net {
        path: String,
        net_name: String,
        #[arg(long)]
        hierarchical: bool,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Count unconnected net segments.
    Unconnected {
        path: String,
        #[arg(long)]
        hierarchical: bool,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Debug, ClapArgs)]
pub struct SchematicQueryCommand {
    #[command(subcommand)]
    pub action: SchematicQueryAction,
}

#[derive(Debug, Subcommand)]
pub enum SymbolLibAction {
    /// Inspect a symbol library or a single symbol within it.
    Inspect {
        path: String,
        symbol: Option<String>,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Set a symbol property in a symbol library.
    SetProperty {
        path: String,
        symbol: String,
        key: String,
        value: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Remove a symbol property from a symbol library.
    RemoveProperty {
        path: String,
        symbol: String,
        key: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Rename a symbol in a symbol library.
    Rename {
        path: String,
        from: String,
        to: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Debug, ClapArgs)]
#[command(
    about = "Inspect or edit a symbol library",
    after_long_help = SYMBOL_LIB_AFTER_LONG_HELP
)]
pub struct SymbolLibCommand {
    #[command(subcommand)]
    pub action: SymbolLibAction,
}

#[derive(Debug, Subcommand)]
pub enum PcbAction {
    /// Inspect a PCB.
    Inspect {
        path: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Query PCB objects.
    Query(PcbQueryCommand),
    /// Set a board property.
    SetProperty {
        path: String,
        key: String,
        value: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a PCB trace segment.
    AddTrace {
        path: String,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        width: f64,
        layer: String,
        net: i32,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Remove a PCB trace segment.
    RemoveTrace {
        path: String,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a via.
    AddVia {
        path: String,
        x: f64,
        y: f64,
        size: f64,
        drill: f64,
        net: i32,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a footprint.
    AddFootprint {
        path: String,
        lib_ref: String,
        x: f64,
        y: f64,
        layer: String,
        reference: String,
        value: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Move a footprint.
    MoveFootprint {
        path: String,
        reference: String,
        x: f64,
        y: f64,
        rotation: Option<f64>,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Remove a footprint.
    RemoveFootprint {
        path: String,
        reference: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Debug, ClapArgs)]
#[command(about = "Inspect or edit a PCB", after_long_help = PCB_AFTER_LONG_HELP)]
pub struct PcbCommand {
    #[command(subcommand)]
    pub action: PcbAction,
}

#[derive(Debug, Subcommand)]
pub enum PcbQueryAction {
    /// Query a footprint by reference.
    Footprint {
        path: String,
        reference: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Debug, ClapArgs)]
pub struct PcbQueryCommand {
    #[command(subcommand)]
    pub action: PcbQueryAction,
}

#[derive(Debug, Subcommand)]
pub enum LibTableAction {
    /// Inspect a library table.
    Inspect {
        path: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Add a library entry.
    Add {
        path: String,
        name: String,
        uri: String,
        #[command(flatten)]
        output: OutputArgs,
    },
    /// Rename a library entry.
    Rename {
        path: String,
        from: String,
        to: String,
        #[command(flatten)]
        output: OutputArgs,
    },
}

#[derive(Debug, ClapArgs)]
#[command(
    about = "Inspect or edit a library table",
    after_long_help = LIB_TABLE_AFTER_LONG_HELP
)]
pub struct LibTableCommand {
    #[command(subcommand)]
    pub action: LibTableAction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_defaults_to_schematic() {
        let args = Args::try_parse_from(["ki", "refresh"]).expect("refresh args should parse");

        match args.command {
            Command::Refresh(refresh) => assert_eq!(refresh.frame, FrameArg::Schematic),
            _ => panic!("expected refresh"),
        }
    }

    #[test]
    fn refresh_accepts_pcb_frame() {
        let args =
            Args::try_parse_from(["ki", "refresh", "--frame", "pcb"]).expect("pcb should parse");

        match args.command {
            Command::Refresh(refresh) => assert_eq!(refresh.frame, FrameArg::Pcb),
            _ => panic!("expected refresh"),
        }
    }

    #[test]
    fn refresh_rejects_unknown_frame() {
        let err = Args::try_parse_from(["ki", "refresh", "--frame", "bogus"])
            .expect_err("unknown frame should fail");

        assert_eq!(err.kind(), clap::error::ErrorKind::InvalidValue);
    }

    #[test]
    fn spice_frame_has_no_document_fallback() {
        assert_eq!(FrameArg::Spice.into_document_type(), None);
    }

    #[test]
    fn parses_schematic_inspect() {
        let args = Args::try_parse_from(["ki", "schematic", "inspect", "a.kicad_sch", "--json"])
            .expect("schematic inspect should parse");
        assert!(matches!(
            args.command,
            Command::Schematic(SchematicCommand {
                action: SchematicAction::Inspect { .. }
            })
        ));
    }

    #[test]
    fn parses_query_net_hierarchical() {
        let args = Args::try_parse_from([
            "ki",
            "schematic",
            "query",
            "net",
            "a.kicad_sch",
            "VCC",
            "--hierarchical",
        ])
        .expect("query net should parse");
        assert!(matches!(
            args.command,
            Command::Schematic(SchematicCommand {
                action: SchematicAction::Query(_)
            })
        ));
    }

    #[test]
    fn parses_update_from_lib_reference() {
        let args = Args::try_parse_from([
            "ki",
            "schematic",
            "update-from-lib",
            "a.kicad_sch",
            "Device",
            "R1",
        ])
        .expect("update-from-lib should parse");
        assert!(matches!(
            args.command,
            Command::Schematic(SchematicCommand {
                action: SchematicAction::UpdateFromLib { .. }
            })
        ));
    }

    #[test]
    fn parses_update_from_lib_all() {
        let args = Args::try_parse_from([
            "ki",
            "schematic",
            "update-from-lib",
            "a.kicad_sch",
            "Device",
            "--all",
        ])
        .expect("update-from-lib --all should parse");
        assert!(matches!(
            args.command,
            Command::Schematic(SchematicCommand {
                action: SchematicAction::UpdateFromLib { all: true, .. }
            })
        ));
    }

    #[test]
    fn parses_update_from_lib_preserve_properties() {
        let args = Args::try_parse_from([
            "ki",
            "schematic",
            "update-from-lib",
            "a.kicad_sch",
            "Device",
            "R1",
            "--preserve-property",
            "MPN",
            "--preserve-property",
            "Manufacturer",
        ])
        .expect("update-from-lib preserve properties should parse");
        assert!(matches!(
            args.command,
            Command::Schematic(SchematicCommand {
                action: SchematicAction::UpdateFromLib {
                    preserve_property,
                    ..
                }
            }) if preserve_property == ["MPN", "Manufacturer"]
        ));
    }

    #[test]
    fn parses_replace_from_lib_preserve_property() {
        let args = Args::try_parse_from([
            "ki",
            "schematic",
            "replace-from-lib",
            "a.kicad_sch",
            "R1",
            "Device",
            "R",
            "--preserve-property",
            "MPN",
        ])
        .expect("replace-from-lib preserve property should parse");
        assert!(matches!(
            args.command,
            Command::Schematic(SchematicCommand {
                action: SchematicAction::ReplaceFromLib {
                    preserve_property,
                    ..
                }
            }) if preserve_property == ["MPN"]
        ));
    }

    #[test]
    fn parses_symbol_lib_optional_symbol() {
        let args =
            Args::try_parse_from(["ki", "symbol-lib", "inspect", "lib.kicad_sym", "Amplifier"])
                .expect("symbol inspect should parse");
        assert!(matches!(
            args.command,
            Command::SymbolLib(SymbolLibCommand {
                action: SymbolLibAction::Inspect { .. }
            })
        ));
    }

    #[test]
    fn parses_move_footprint_rotation() {
        let args = Args::try_parse_from([
            "ki",
            "pcb",
            "move-footprint",
            "board.kicad_pcb",
            "U1",
            "10",
            "20",
            "90",
        ])
        .expect("move-footprint should parse");
        assert!(matches!(
            args.command,
            Command::Pcb(PcbCommand {
                action: PcbAction::MoveFootprint { .. }
            })
        ));
    }
}
