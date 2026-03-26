use clap::{Parser, Subcommand, ValueEnum};
use kicad_ipc_rs::{DocumentType, EditorFrameType};

#[derive(Debug, Parser)]
#[command(name = "ki", about = "KiCad IPC CLI")]
pub struct Args {
    /// Override the KiCad IPC socket URI/path.
    #[arg(long, global = true)]
    pub socket: Option<String>,

    /// Override the KiCad instance token.
    #[arg(long, global = true)]
    pub token: Option<String>,

    /// Set the client name sent to KiCad.
    #[arg(long, global = true)]
    pub client_name: Option<String>,

    /// IPC request timeout in milliseconds.
    #[arg(long, global = true, default_value_t = 3_000)]
    pub timeout_ms: u64,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Refresh a KiCad editor frame.
    Refresh(RefreshArgs),
}

#[derive(Debug, clap::Args)]
pub struct RefreshArgs {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_defaults_to_schematic() {
        let args = Args::try_parse_from(["ki", "refresh"]).expect("refresh args should parse");

        match args.command {
            Command::Refresh(refresh) => {
                assert_eq!(refresh.frame, FrameArg::Schematic);
            }
        }
    }

    #[test]
    fn refresh_accepts_pcb_frame() {
        let args =
            Args::try_parse_from(["ki", "refresh", "--frame", "pcb"]).expect("pcb should parse");

        match args.command {
            Command::Refresh(refresh) => {
                assert_eq!(refresh.frame, FrameArg::Pcb);
            }
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
    fn parses_silent_flag() {
        let args =
            Args::try_parse_from(["ki", "refresh", "--silent"]).expect("silent should parse");

        match args.command {
            Command::Refresh(refresh) => assert!(refresh.silent),
        }
    }
}
