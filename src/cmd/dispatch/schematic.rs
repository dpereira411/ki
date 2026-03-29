use crate::cli::{SchematicAction, SchematicCommand, SchematicQueryAction};
use crate::error::KiError;

pub fn run(schematic_cmd: SchematicCommand) -> Result<(), KiError> {
    match schematic_cmd.action {
        SchematicAction::Inspect { path, output } => {
            super::super::schematic::inspect(&path, &super::output_flags(&output, false))
        }
        SchematicAction::SetProperty {
            path,
            reference,
            key,
            value,
            output,
        } => {
            super::super::schematic::set_property(
                &path,
                &reference,
                &key,
                &value,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::RemoveProperty {
            path,
            reference,
            key,
            output,
        } => {
            super::super::schematic::remove_property(
                &path,
                &reference,
                &key,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::AddSymbol {
            path,
            lib_id,
            reference,
            value,
            x,
            y,
            output,
        } => {
            super::super::schematic::add_symbol(
                &path,
                &lib_id,
                &reference,
                &value,
                x,
                y,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::RemoveSymbol {
            path,
            reference,
            output,
        } => {
            super::super::schematic::remove_symbol(
                &path,
                &reference,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::Rename {
            path,
            reference,
            new_lib_id,
            output,
        } => {
            super::super::schematic::rename(
                &path,
                &reference,
                &new_lib_id,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::AddWire {
            path,
            x1,
            y1,
            x2,
            y2,
            output,
        } => {
            super::super::schematic::add_wire(
                &path,
                x1,
                y1,
                x2,
                y2,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::RemoveWire {
            path,
            x1,
            y1,
            x2,
            y2,
            output,
        } => {
            super::super::schematic::remove_wire(
                &path,
                x1,
                y1,
                x2,
                y2,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::AddLabel {
            path,
            text,
            x,
            y,
            angle,
            output,
        } => {
            super::super::schematic::add_label(
                &path,
                &text,
                x,
                y,
                angle,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::AddGlobalLabel {
            path,
            text,
            shape,
            x,
            y,
            angle,
            output,
        } => {
            super::super::schematic::add_global_label(
                &path,
                &text,
                &shape,
                x,
                y,
                angle,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::RemoveLabel { path, name, output } => {
            super::super::schematic::remove_label(
                &path,
                &name,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::RenameLabel {
            path,
            name,
            new_name,
            output,
        } => {
            super::super::schematic::rename_label(
                &path,
                &name,
                &new_name,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::AddJunction { path, x, y, output } => {
            super::super::schematic::add_junction(
                &path,
                x,
                y,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::AddNoConnect { path, x, y, output } => {
            super::super::schematic::add_no_connect(
                &path,
                x,
                y,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::ForkSymbol {
            path,
            reference,
            library_name,
            target_symbol_name,
            overwrite,
            output,
        } => {
            super::super::schematic::fork_symbol(
                &path,
                &reference,
                &library_name,
                &target_symbol_name,
                overwrite,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::PushToLib {
            path,
            reference,
            library_name,
            output,
        } => {
            super::super::schematic::push_to_lib(
                &path,
                &reference,
                &library_name,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::ReplaceFromLib {
            path,
            reference,
            library_name,
            symbol_name,
            override_value,
            preserve_property,
            output,
        } => {
            super::super::schematic::replace_from_lib(
                &path,
                &reference,
                &library_name,
                &symbol_name,
                override_value,
                &preserve_property,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::UpdateFromLib {
            path,
            library_name,
            reference,
            all,
            override_value,
            preserve_property,
            output,
        } => {
            if !all && reference.is_none() {
                return Err(KiError::Message(
                    "missing argument <reference> or --all".to_string(),
                ));
            }
            super::super::schematic::update_from_lib(
                &path,
                &library_name,
                reference.as_deref(),
                all,
                override_value,
                &preserve_property,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
        SchematicAction::Erc {
            path,
            output,
            units,
            severity_all,
            severity_error,
            severity_warning,
            severity_exclusions,
            exit_code_violations,
            format,
        } => super::super::schematic::run_erc(
            &path,
            output.as_deref(),
            &units,
            severity_all,
            severity_error,
            severity_warning,
            severity_exclusions,
            exit_code_violations,
            &super::output_flags(&format, false),
        ),
        SchematicAction::Query(query) => match query.action {
            SchematicQueryAction::Component {
                path,
                reference,
                output,
            } => {
                super::super::schematic::query_component(
                    &path,
                    &reference,
                    &super::output_flags(&output, false),
                )?;
                Ok(())
            }
            SchematicQueryAction::Net {
                path,
                net_name,
                hierarchical,
                output,
            } => {
                super::super::schematic::query_net(
                    &path,
                    &net_name,
                    &super::output_flags(&output, hierarchical),
                )?;
                Ok(())
            }
            SchematicQueryAction::Unconnected {
                path,
                hierarchical,
                output,
            } => {
                super::super::schematic::query_unconnected(
                    &path,
                    &super::output_flags(&output, hierarchical),
                )?;
                Ok(())
            }
        },
        SchematicAction::CheckIntent {
            path,
            intent,
            output,
        } => {
            super::super::schematic::check_intent(
                &path,
                &intent,
                &super::output_flags(&output, false),
            )?;
            Ok(())
        }
    }
}
