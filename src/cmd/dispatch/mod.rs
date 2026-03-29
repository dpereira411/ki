mod lib_table;
mod pcb;
mod project;
mod schematic;
mod symbol;

use crate::cli::{Command, OutputArgs};
use crate::error::KiError;
use crate::output::Flags;

fn output_flags(output: &OutputArgs, _hierarchical: bool) -> Flags {
    Flags::new(output.json, output.diagnostics)
}

pub fn run(command: Command) -> Result<(), KiError> {
    match command {
        Command::Refresh(_) => Err(KiError::Message("refresh is handled in main".to_string())),
        Command::Extract(extract_args) => super::extract::run(&extract_args),
        Command::Project(project_cmd) => project::run(project_cmd),
        Command::Schematic(schematic_cmd) => schematic::run(schematic_cmd),
        Command::SymbolLib(symbol_lib_cmd) => symbol::run(symbol_lib_cmd),
        Command::Pcb(pcb_cmd) => pcb::run(pcb_cmd),
        Command::LibTable(lib_table_cmd) => lib_table::run(lib_table_cmd),
    }
}
