use serde::Serialize;

#[derive(Debug, Clone)]
pub struct ExtractedNetlist {
    pub source: String,
    pub project: Option<String>,
    pub tool: Option<String>,
    pub version: Option<i32>,
    pub sheet_root: Option<String>,
    pub components: Vec<Component>,
    pub lib_parts: Vec<LibPart>,
    pub nets: Vec<Net>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibPart {
    pub lib: String,
    pub part: String,
    pub description: Option<String>,
    pub docs: Option<String>,
    pub footprints: Vec<String>,
    pub fields: Vec<Field>,
    pub pins: Vec<LibPin>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibPin {
    pub num: String,
    pub name: Option<String>,
    pub electrical_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Component {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub lib: Option<String>,
    pub part: Option<String>,
    pub value: Option<String>,
    pub footprint: Option<String>,
    pub datasheet: Option<String>,
    pub sheet_path: Option<String>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Property {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Net {
    pub code: i32,
    pub name: String,
    pub labels: Vec<NetLabel>,
    pub nodes: Vec<NetNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetLabel {
    pub text: String,
    pub x: f64,
    pub y: f64,
    #[serde(rename = "type")]
    pub label_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetNode {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub pin: String,
    pub pin_function: Option<String>,
    pub pin_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentPin {
    pub num: String,
    pub net: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractComponent {
    #[serde(rename = "ref")]
    pub ref_: String,
    pub lib_part_id: Option<String>,
    pub value: Option<String>,
    pub footprint: Option<String>,
    pub datasheet: Option<String>,
    pub sheet_path: Option<String>,
    pub properties: Vec<Property>,
    pub pins: Vec<ComponentPin>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractLibPin {
    pub num: String,
    pub name: Option<String>,
    pub electrical_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractLibPart {
    pub id: String,
    pub lib: String,
    pub part: String,
    pub description: Option<String>,
    pub documentation: Option<String>,
    pub footprint_filters: Vec<String>,
    pub fields: Vec<Field>,
    pub pins: Vec<ExtractLibPin>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractNetNode {
    pub component_ref: String,
    pub pin_num: String,
    pub pin_name: Option<String>,
    pub pin_electrical_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractNet {
    pub code: i32,
    pub name: String,
    pub labels: Vec<NetLabel>,
    pub nodes: Vec<ExtractNetNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceInfo {
    pub schematic: String,
    pub project: Option<String>,
    pub tool: Option<String>,
    pub version: Option<i32>,
    pub root_sheet_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtractDoc {
    pub schema_version: u32,
    pub source: SourceInfo,
    pub lib_parts: Vec<ExtractLibPart>,
    pub components: Vec<ExtractComponent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nets: Option<Vec<ExtractNet>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<serde_json::Value>>,
}
