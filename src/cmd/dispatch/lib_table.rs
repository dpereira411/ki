use crate::cli::{LibTableAction, LibTableCommand};
use crate::error::KiError;

pub fn run(lib_table_cmd: LibTableCommand) -> Result<(), KiError> {
    match lib_table_cmd.action {
        LibTableAction::Inspect { path, output } => {
            super::super::lib_table::inspect(&path, &super::output_flags(&output, false))
        }
        LibTableAction::Add {
            path,
            name,
            uri,
            output,
        } => super::super::lib_table::add(&path, &name, &uri, &super::output_flags(&output, false)),
        LibTableAction::Rename {
            path,
            from,
            to,
            output,
        } => {
            super::super::lib_table::rename(&path, &from, &to, &super::output_flags(&output, false))
        }
    }
}
