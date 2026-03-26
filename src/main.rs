mod cli;
mod error;

use std::process::ExitCode;
use std::time::Duration;

use clap::Parser;
use cli::{Args, Command};
use kicad_ipc_rs::{KiCadClientBlocking, KiCadError};

fn main() -> ExitCode {
    let args = Args::parse();
    let silent = match &args.command {
        Command::Refresh(refresh) => refresh.silent,
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

fn run(args: Args) -> Result<(), RunError> {
    let silent = match &args.command {
        Command::Refresh(refresh) => refresh.silent,
    };

    let mut builder =
        KiCadClientBlocking::builder().timeout(Duration::from_millis(args.timeout_ms));
    if let Some(socket) = args.socket {
        builder = builder.socket_path(socket);
    }
    if let Some(token) = args.token {
        builder = builder.token(token);
    }
    if let Some(client_name) = args.client_name {
        builder = builder.client_name(client_name);
    }

    let client = builder
        .connect()
        .map_err(|error| RunError::from((silent, error)))?;

    match args.command {
        Command::Refresh(refresh) => {
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
        }
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
