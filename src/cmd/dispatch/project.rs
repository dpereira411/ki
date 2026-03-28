use crate::cli::{ProjectAction, ProjectCommand};
use crate::error::KiError;

pub fn run(project_cmd: ProjectCommand) -> Result<(), KiError> {
    match project_cmd.action {
        ProjectAction::Open { path, output } => {
            super::super::project::open(&path, &super::output_flags(&output, false))
        }
        ProjectAction::Validate { path, output } => {
            super::super::project::validate(&path, &super::output_flags(&output, false))
        }
    }
}
