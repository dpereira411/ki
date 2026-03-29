use std::collections::BTreeMap;

use super::model::{
    ComponentPin, ExtractComponent, ExtractDoc, ExtractLibPart, ExtractLibPin, ExtractNet,
    ExtractNetNode, ExtractReport, SourceInfo,
};

pub struct RenderOptions {
    pub include_nets: bool,
    pub include_diagnostics: bool,
}

pub fn render_doc(report: &ExtractReport, options: &RenderOptions) -> ExtractDoc {
    let netlist = &report.netlist;
    let mut component_net_lookup: BTreeMap<(String, String), String> = BTreeMap::new();
    for net in &netlist.nets {
        for node in &net.nodes {
            component_net_lookup.insert((node.ref_.clone(), node.pin.clone()), net.name.clone());
        }
    }
    let lib_part_lookup = netlist
        .lib_parts
        .iter()
        .map(|lib_part| ((lib_part.lib.clone(), lib_part.part.clone()), lib_part))
        .collect::<BTreeMap<_, _>>();

    ExtractDoc {
        schema_version: super::build::EXTRACT_SCHEMA_VERSION,
        source: SourceInfo {
            schematic: netlist.source.clone(),
            project: netlist.project.clone(),
            tool: netlist.tool.clone(),
            version: netlist.version,
            root_sheet_path: netlist.sheet_root.clone(),
        },
        lib_parts: netlist
            .lib_parts
            .iter()
            .map(|lib_part| ExtractLibPart {
                id: format!("{}:{}", lib_part.lib, lib_part.part),
                description: lib_part.description.clone(),
                datasheet: lib_part.docs.clone(),
                footprint_filters: lib_part.footprints.clone(),
                fields: lib_part.fields.clone(),
                pins: lib_part
                    .pins
                    .iter()
                    .map(|pin| ExtractLibPin {
                        num: pin.num.clone(),
                        name: pin.name.clone(),
                        electrical_kind: pin.electrical_type.clone(),
                    })
                    .collect(),
            })
            .collect(),
        components: netlist
            .components
            .iter()
            .map(|component| {
                let lib_part_id = match (&component.lib, &component.part) {
                    (Some(lib), Some(part)) => Some(format!("{lib}:{part}")),
                    _ => None,
                };
                let pins = match (&component.lib, &component.part) {
                    (Some(lib), Some(part)) => lib_part_lookup
                        .get(&(lib.clone(), part.clone()))
                        .map(|lib_part| {
                            lib_part
                                .pins
                                .iter()
                                .map(|pin| ComponentPin {
                                    num: pin.num.clone(),
                                    net: component_net_lookup
                                        .get(&(component.ref_.clone(), pin.num.clone()))
                                        .cloned(),
                                })
                                .collect()
                        })
                        .unwrap_or_default(),
                    _ => Vec::new(),
                };

                ExtractComponent {
                    ref_: component.ref_.clone(),
                    lib_part_id,
                    value: component.value.clone(),
                    footprint: component.footprint.clone(),
                    datasheet: component.datasheet.clone(),
                    sheet_path: component.sheet_path.clone(),
                    properties: component.properties.clone(),
                    pins,
                }
            })
            .collect(),
        nets: options.include_nets.then(|| {
            netlist
                .nets
                .iter()
                .map(|net| ExtractNet {
                    code: net.code,
                    name: net.name.clone(),
                    labels: net.labels.clone(),
                    nodes: net
                        .nodes
                        .iter()
                        .map(|node| ExtractNetNode {
                            component_ref: node.ref_.clone(),
                            pin_num: node.pin.clone(),
                            pin_name: node.pin_function.clone(),
                            pin_electrical_kind: node.pin_type.clone(),
                        })
                        .collect(),
                })
                .collect()
        }),
        diagnostics: options
            .include_diagnostics
            .then(|| report.diagnostics.clone()),
    }
}
