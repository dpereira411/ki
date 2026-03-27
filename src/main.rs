mod cli;
mod cmd;
mod error;
mod output;

use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use cli::{
    Args, Command, LibTableAction, OutputArgs, PcbAction, PcbQueryAction, ProjectAction,
    RefreshArgs, SchematicAction, SchematicQueryAction, SymbolLibAction,
};
use kicad_ipc_rs::{KiCadClientBlocking, KiCadError};
use output::Flags;

fn main() -> ExitCode {
    let raw_args: Vec<_> = std::env::args_os().collect();
    if raw_args.len() == 1 {
        cli::print_root_usage();
        return ExitCode::from(2);
    }
    if raw_args.len() == 2 && (raw_args[1] == "--help" || raw_args[1] == "-h") {
        cli::print_root_usage();
        return ExitCode::SUCCESS;
    }

    let args = Args::parse();
    let silent = match &args.command {
        Command::Refresh(refresh) => refresh.silent,
        _ => false,
    };

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            if should_exit_success_on_error(silent) {
                return ExitCode::SUCCESS;
            }
            if !err.silent {
                for line in error::render_kicad_error(&err.error) {
                    eprintln!("{line}");
                }
            }
            ExitCode::from(1)
        }
    }
}

struct RunError {
    silent: bool,
    error: KiCadError,
}

impl From<(bool, KiCadError)> for RunError {
    fn from((silent, error): (bool, KiCadError)) -> Self {
        Self { silent, error }
    }
}

fn output_flags(output: &OutputArgs, hierarchical: bool) -> Flags {
    Flags::new(output.json, output.diagnostics, hierarchical)
}

fn run(args: Args) -> Result<(), RunError> {
    match args.command {
        Command::Refresh(refresh) => return run_refresh(refresh),
        Command::Project(project) => match project.action {
            ProjectAction::Open { path, output } => {
                cmd::project::open(&path, &output_flags(&output, false))
            }
            ProjectAction::Validate { path, output } => {
                cmd::project::validate(&path, &output_flags(&output, false))
            }
        },
        Command::Schematic(schematic) => match schematic.action {
            SchematicAction::Inspect { path, output } => {
                cmd::schematic::inspect(&path, &output_flags(&output, false))
            }
            SchematicAction::SetProperty {
                path,
                reference,
                key,
                value,
                output,
            } => cmd::schematic::set_property(
                &path,
                &reference,
                &key,
                &value,
                &output_flags(&output, false),
            ),
            SchematicAction::RemoveProperty {
                path,
                reference,
                key,
                output,
            } => cmd::schematic::remove_property(
                &path,
                &reference,
                &key,
                &output_flags(&output, false),
            ),
            SchematicAction::AddSymbol {
                path,
                lib_id,
                reference,
                value,
                x,
                y,
                output,
            } => cmd::schematic::add_symbol(
                &path,
                &lib_id,
                &reference,
                &value,
                &x,
                &y,
                &output_flags(&output, false),
            ),
            SchematicAction::RemoveSymbol {
                path,
                reference,
                output,
            } => cmd::schematic::remove_symbol(&path, &reference, &output_flags(&output, false)),
            SchematicAction::Rename {
                path,
                reference,
                new_lib_id,
                output,
            } => cmd::schematic::rename(
                &path,
                &reference,
                &new_lib_id,
                &output_flags(&output, false),
            ),
            SchematicAction::AddWire {
                path,
                x1,
                y1,
                x2,
                y2,
                output,
            } => cmd::schematic::add_wire(&path, &x1, &y1, &x2, &y2, &output_flags(&output, false)),
            SchematicAction::RemoveWire {
                path,
                x1,
                y1,
                x2,
                y2,
                output,
            } => cmd::schematic::remove_wire(
                &path,
                &x1,
                &y1,
                &x2,
                &y2,
                &output_flags(&output, false),
            ),
            SchematicAction::AddLabel {
                path,
                text,
                x,
                y,
                angle,
                output,
            } => cmd::schematic::add_label(
                &path,
                &text,
                &x,
                &y,
                &angle,
                &output_flags(&output, false),
            ),
            SchematicAction::AddGlobalLabel {
                path,
                text,
                shape,
                x,
                y,
                angle,
                output,
            } => cmd::schematic::add_global_label(
                &path,
                &text,
                &shape,
                &x,
                &y,
                &angle,
                &output_flags(&output, false),
            ),
            SchematicAction::AddJunction { path, x, y, output } => {
                cmd::schematic::add_junction(&path, &x, &y, &output_flags(&output, false))
            }
            SchematicAction::AddNoConnect { path, x, y, output } => {
                cmd::schematic::add_no_connect(&path, &x, &y, &output_flags(&output, false))
            }
            SchematicAction::ForkSymbol {
                path,
                reference,
                library_name,
                target_symbol_name,
                overwrite,
                output,
            } => cmd::schematic::fork_symbol(
                &path,
                &reference,
                &library_name,
                &target_symbol_name,
                overwrite,
                &output_flags(&output, false),
            ),
            SchematicAction::PushToLib {
                path,
                reference,
                library_name,
                output,
            } => cmd::schematic::push_to_lib(
                &path,
                &reference,
                &library_name,
                &output_flags(&output, false),
            ),
            SchematicAction::ReplaceFromLib {
                path,
                reference,
                library_name,
                symbol_name,
                override_value,
                output,
            } => cmd::schematic::replace_from_lib(
                &path,
                &reference,
                &library_name,
                &symbol_name,
                override_value,
                &output_flags(&output, false),
            ),
            SchematicAction::UpdateFromLib {
                path,
                library_name,
                reference,
                all,
                override_value,
                output,
            } => {
                if !all && reference.is_none() {
                    output::fatal_error("missing argument <reference> or --all");
                }
                cmd::schematic::update_from_lib(
                    &path,
                    &library_name,
                    reference.as_deref(),
                    all,
                    override_value,
                    &output_flags(&output, false),
                )
            }
            SchematicAction::Query(query) => match query.action {
                SchematicQueryAction::Component {
                    path,
                    reference,
                    output,
                } => cmd::schematic::query_component(
                    &path,
                    &reference,
                    &output_flags(&output, false),
                ),
                SchematicQueryAction::Net {
                    path,
                    net_name,
                    hierarchical,
                    output,
                } => cmd::schematic::query_net(
                    &path,
                    &net_name,
                    &output_flags(&output, hierarchical),
                ),
                SchematicQueryAction::Unconnected {
                    path,
                    hierarchical,
                    output,
                } => cmd::schematic::query_unconnected(&path, &output_flags(&output, hierarchical)),
            },
            SchematicAction::CheckIntent {
                path,
                intent,
                output,
            } => cmd::schematic::check_intent(&path, &intent, &output_flags(&output, false)),
        },
        Command::SymbolLib(symbol_lib) => match symbol_lib.action {
            SymbolLibAction::Inspect {
                path,
                symbol,
                output,
            } => cmd::symbol::inspect(&path, symbol.as_deref(), &output_flags(&output, false)),
            SymbolLibAction::SetProperty {
                path,
                symbol,
                key,
                value,
                output,
            } => cmd::symbol::set_property(
                &path,
                &symbol,
                &key,
                &value,
                &output_flags(&output, false),
            ),
            SymbolLibAction::RemoveProperty {
                path,
                symbol,
                key,
                output,
            } => cmd::symbol::remove_property(&path, &symbol, &key, &output_flags(&output, false)),
            SymbolLibAction::Rename {
                path,
                from,
                to,
                output,
            } => cmd::symbol::rename(&path, &from, &to, &output_flags(&output, false)),
        },
        Command::Pcb(pcb) => match pcb.action {
            PcbAction::Inspect { path, output } => {
                cmd::pcb::inspect(&path, &output_flags(&output, false))
            }
            PcbAction::Query(query) => match query.action {
                PcbQueryAction::Footprint {
                    path,
                    reference,
                    output,
                } => cmd::pcb::query_footprint(&path, &reference, &output_flags(&output, false)),
            },
            PcbAction::SetProperty {
                path,
                key,
                value,
                output,
            } => cmd::pcb::set_property(&path, &key, &value, &output_flags(&output, false)),
            PcbAction::AddTrace {
                path,
                x1,
                y1,
                x2,
                y2,
                width,
                layer,
                net,
                output,
            } => cmd::pcb::add_trace(
                &path,
                &x1,
                &y1,
                &x2,
                &y2,
                &width,
                &layer,
                &net,
                &output_flags(&output, false),
            ),
            PcbAction::RemoveTrace {
                path,
                x1,
                y1,
                x2,
                y2,
                output,
            } => cmd::pcb::remove_trace(&path, &x1, &y1, &x2, &y2, &output_flags(&output, false)),
            PcbAction::AddVia {
                path,
                x,
                y,
                size,
                drill,
                net,
                output,
            } => cmd::pcb::add_via(
                &path,
                &x,
                &y,
                &size,
                &drill,
                &net,
                &output_flags(&output, false),
            ),
            PcbAction::AddFootprint {
                path,
                lib_ref,
                x,
                y,
                layer,
                reference,
                value,
                output,
            } => cmd::pcb::add_footprint(
                &path,
                &lib_ref,
                &x,
                &y,
                &layer,
                &reference,
                &value,
                &output_flags(&output, false),
            ),
            PcbAction::MoveFootprint {
                path,
                reference,
                x,
                y,
                rotation,
                output,
            } => cmd::pcb::move_footprint(
                &path,
                &reference,
                &x,
                &y,
                rotation.as_deref(),
                &output_flags(&output, false),
            ),
            PcbAction::RemoveFootprint {
                path,
                reference,
                output,
            } => cmd::pcb::remove_footprint(&path, &reference, &output_flags(&output, false)),
        },
        Command::LibTable(lib_table) => match lib_table.action {
            LibTableAction::Inspect { path, output } => {
                cmd::lib_table::inspect(&path, &output_flags(&output, false))
            }
            LibTableAction::Add {
                path,
                name,
                uri,
                output,
            } => cmd::lib_table::add(&path, &name, &uri, &output_flags(&output, false)),
            LibTableAction::Rename {
                path,
                from,
                to,
                output,
            } => cmd::lib_table::rename(&path, &from, &to, &output_flags(&output, false)),
        },
    };

    Ok(())
}

fn run_refresh(refresh: RefreshArgs) -> Result<(), RunError> {
    let mut builder =
        KiCadClientBlocking::builder().timeout(Duration::from_millis(refresh.timeout_ms));
    if let Some(socket) = &refresh.socket {
        builder = builder.socket_path(socket);
    }
    if let Some(token) = &refresh.token {
        builder = builder.token(token);
    }
    if let Some(client_name) = &refresh.client_name {
        builder = builder.client_name(client_name);
    }

    let client = builder
        .connect()
        .map_err(|error| RunError::from((refresh.silent, error)))?;

    let frame = refresh.frame.into_editor_frame();
    match client.refresh_editor(frame) {
        Ok(()) => {
            if !refresh.silent {
                println!("refresh_editor=ok frame={frame}");
            }
        }
        Err(err) if is_refresh_unhandled(&err) => {
            if let Some(document_type) = refresh.frame.into_document_type() {
                client
                    .revert_document_for_type(document_type)
                    .map_err(|error| RunError::from((refresh.silent, error)))?;
                if !refresh.silent {
                    println!("revert_document=ok document={document_type} frame={frame}");
                }
            } else {
                return Err(RunError::from((refresh.silent, err)));
            }
        }
        Err(err) => return Err(RunError::from((refresh.silent, err))),
    }

    Ok(())
}

fn should_exit_success_on_error(silent: bool) -> bool {
    silent
}

fn is_refresh_unhandled(err: &KiCadError) -> bool {
    matches!(
        err,
        KiCadError::ApiStatus { code, message }
            if code == "AS_UNHANDLED"
                && message.contains("kiapi.common.commands.RefreshEditor")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_refresh_unhandled_shape() {
        assert!(is_refresh_unhandled(&KiCadError::ApiStatus {
            code: "AS_UNHANDLED".to_string(),
            message: "no handler available for request of type kiapi.common.commands.RefreshEditor"
                .to_string(),
        }));
    }

    #[test]
    fn ignores_other_unhandled_calls() {
        assert!(!is_refresh_unhandled(&KiCadError::ApiStatus {
            code: "AS_UNHANDLED".to_string(),
            message:
                "no handler available for request of type kiapi.common.commands.GetOpenDocuments"
                    .to_string(),
        }));
    }

    #[test]
    fn silent_mode_exits_success_on_error() {
        assert!(should_exit_success_on_error(true));
        assert!(!should_exit_success_on_error(false));
    }
}
