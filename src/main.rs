mod cli;
mod cmd;
mod error;
mod extract;
mod output;
mod schematic;

use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use cli::{Args, Command, RefreshArgs};
use kicad_ipc_rs::{KiCadClientBlocking, KiCadError};

use error::KiError;

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
            if matches!(err.error, KiError::Validation) {
                return ExitCode::from(1);
            }
            if is_schematic_load_failure(&err.error) {
                if !err.silent {
                    for line in error::render_error(&err.error) {
                        eprintln!("{line}");
                    }
                }
                return ExitCode::from(3);
            }
            if !err.silent {
                for line in error::render_error(&err.error) {
                    eprintln!("{line}");
                }
            }
            ExitCode::from(2)
        }
    }
}

struct RunError {
    silent: bool,
    error: KiError,
}

impl From<(bool, KiError)> for RunError {
    fn from((silent, error): (bool, KiError)) -> Self {
        Self { silent, error }
    }
}

fn run(args: Args) -> Result<(), RunError> {
    match args.command {
        Command::Refresh(refresh) => run_refresh(refresh)?,
        command => cmd::run(command).map_err(|error| RunError::from((false, error)))?,
    }

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
        .map_err(|error| RunError::from((refresh.silent, KiError::KiCad(error))))?;

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
                    .map_err(|error| RunError::from((refresh.silent, KiError::KiCad(error))))?;
                if !refresh.silent {
                    println!("revert_document=ok document={document_type} frame={frame}");
                }
            } else {
                return Err(RunError::from((refresh.silent, KiError::KiCad(err))));
            }
        }
        Err(err) => return Err(RunError::from((refresh.silent, KiError::KiCad(err)))),
    }

    Ok(())
}

fn should_exit_success_on_error(silent: bool) -> bool {
    silent
}

fn is_schematic_load_failure(err: &KiError) -> bool {
    match err {
        KiError::Message(msg) => msg == "Failed to load schematic",
        KiError::Lines(lines) => lines.iter().any(|line| line == "Failed to load schematic"),
        _ => false,
    }
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
