use crate::cli::{PcbAction, PcbCommand, PcbQueryAction};
use crate::error::KiError;

pub fn run(pcb_cmd: PcbCommand) -> Result<(), KiError> {
    match pcb_cmd.action {
        PcbAction::Inspect { path, output } => {
            super::super::pcb::inspect(&path, &super::output_flags(&output, false))
        }
        PcbAction::Query(query) => match query.action {
            PcbQueryAction::Footprint {
                path,
                reference,
                output,
            } => super::super::pcb::query_footprint(
                &path,
                &reference,
                &super::output_flags(&output, false),
            ),
        },
        PcbAction::SetProperty {
            path,
            key,
            value,
            output,
        } => super::super::pcb::set_property(
            &path,
            &key,
            &value,
            &super::output_flags(&output, false),
        ),
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
        } => super::super::pcb::add_trace(
            &path,
            x1,
            y1,
            x2,
            y2,
            width,
            &layer,
            net,
            &super::output_flags(&output, false),
        ),
        PcbAction::RemoveTrace {
            path,
            x1,
            y1,
            x2,
            y2,
            output,
        } => super::super::pcb::remove_trace(
            &path,
            x1,
            y1,
            x2,
            y2,
            &super::output_flags(&output, false),
        ),
        PcbAction::AddVia {
            path,
            x,
            y,
            size,
            drill,
            net,
            output,
        } => super::super::pcb::add_via(
            &path,
            x,
            y,
            size,
            drill,
            net,
            &super::output_flags(&output, false),
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
        } => super::super::pcb::add_footprint(
            &path,
            &lib_ref,
            x,
            y,
            &layer,
            &reference,
            &value,
            &super::output_flags(&output, false),
        ),
        PcbAction::MoveFootprint {
            path,
            reference,
            x,
            y,
            rotation,
            output,
        } => super::super::pcb::move_footprint(
            &path,
            &reference,
            x,
            y,
            rotation,
            &super::output_flags(&output, false),
        ),
        PcbAction::RemoveFootprint {
            path,
            reference,
            output,
        } => super::super::pcb::remove_footprint(
            &path,
            &reference,
            &super::output_flags(&output, false),
        ),
    }
}
