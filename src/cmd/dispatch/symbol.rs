use crate::cli::{SymbolLibAction, SymbolLibCommand};
use crate::error::KiError;

pub fn run(symbol_lib_cmd: SymbolLibCommand) -> Result<(), KiError> {
    match symbol_lib_cmd.action {
        SymbolLibAction::Inspect {
            path,
            symbol,
            output,
        } => super::super::symbol::inspect(
            &path,
            symbol.as_deref(),
            &super::output_flags(&output, false),
        ),
        SymbolLibAction::SetProperty {
            path,
            symbol,
            key,
            value,
            output,
        } => super::super::symbol::set_property(
            &path,
            &symbol,
            &key,
            &value,
            &super::output_flags(&output, false),
        ),
        SymbolLibAction::RemoveProperty {
            path,
            symbol,
            key,
            output,
        } => super::super::symbol::remove_property(
            &path,
            &symbol,
            &key,
            &super::output_flags(&output, false),
        ),
        SymbolLibAction::Rename {
            path,
            from,
            to,
            output,
        } => super::super::symbol::rename(&path, &from, &to, &super::output_flags(&output, false)),
    }
}
