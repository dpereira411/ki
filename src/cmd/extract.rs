use std::collections::BTreeSet;
use std::path::Path;

use crate::cli::ExtractArgs;
use crate::error::KiError;
use crate::extract::{render, sym_lib};

pub fn run(args: &ExtractArgs) -> Result<(), KiError> {
    let input = Path::new(&args.input);
    if input.extension().and_then(|s| s.to_str()) != Some("kicad_sch") {
        return Err(KiError::Message(format!(
            "unsupported input extension for {:?} (expected .kicad_sch)",
            input
        )));
    }

    let mut netlist =
        render::extract_from_schematic(&args.input).map_err(|err| KiError::Message(err))?;
    let mut sym_lib_paths = Vec::new();
    let mut seen = BTreeSet::new();

    for path in sym_lib::discover_project_sym_libs(input, args.verbose) {
        if seen.insert(path.clone()) {
            sym_lib_paths.push(path);
        }
    }
    for path in &args.sym_lib {
        if seen.insert(path.clone()) {
            sym_lib_paths.push(path.clone());
        }
    }

    sym_lib::enrich(&mut netlist, &sym_lib_paths)
        .map_err(|err| KiError::Message(err.to_string()))?;

    let doc = render::render_doc(
        &netlist,
        &render::RenderOptions {
            include_nets: args.include_nets,
            include_diagnostics: args.include_diagnostics,
        },
    );
    let json = if args.pretty {
        serde_json::to_string_pretty(&doc)
    } else {
        serde_json::to_string(&doc)
    }
    .map_err(|err| KiError::Message(err.to_string()))?;

    if let Some(output) = &args.output {
        std::fs::write(output, format!("{json}\n")).map_err(|err| {
            KiError::Message(format!("failed to write output to {output:?}: {err}"))
        })?;
    } else {
        println!("{json}");
    }

    Ok(())
}
