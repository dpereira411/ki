use serde::{Deserialize, Serialize};

use crate::error::KiError;
use crate::output::{self, CommandResponse, Flags, SCHEMA_VERSION};
use crate::schematic::render::{parse_schema, resolve_nets};

#[derive(Debug, Deserialize)]
struct Intent {
    pins: Vec<PinIntent>,
}

#[derive(Debug, Deserialize)]
struct PinIntent {
    reference: String,
    pin: String,
    net: String,
}

#[derive(Serialize)]
struct IntentResponse {
    schema_version: u32,
    ok: bool,
    mismatches: Vec<String>,
}

impl CommandResponse for IntentResponse {
    fn render_text(&self) {
        if self.ok {
            println!("ok: intent matches schematic");
        } else {
            println!("error: intent mismatches found:");
            for m in &self.mismatches {
                println!("  - {m}");
            }
        }
    }
}

pub fn check_intent(path: &str, intent_path: &str, flags: &Flags) -> Result<(), KiError> {
    let schema = parse_schema(path).map_err(|e: String| KiError::Message(e))?;
    let nets = resolve_nets(&schema);

    let intent_json = std::fs::read_to_string(intent_path).map_err(|e| KiError::Io(e))?;
    let intent: Intent =
        serde_json::from_str(&intent_json).map_err(|e| KiError::Message(e.to_string()))?;

    let mut mismatches = Vec::new();

    for pin_intent in intent.pins {
        let found_net = nets.iter().find(|net| {
            net.nodes
                .iter()
                .any(|node| node.reference == pin_intent.reference && node.pin == pin_intent.pin)
        });

        match found_net {
            Some(n) if n.name == pin_intent.net => {}
            Some(n) => mismatches.push(format!(
                "{}.pin {} net mismatch: expected {}, found {}",
                pin_intent.reference, pin_intent.pin, pin_intent.net, n.name
            )),
            None => mismatches.push(format!(
                "{}.pin {} not found in schematic",
                pin_intent.reference, pin_intent.pin
            )),
        }
    }

    let ok = mismatches.is_empty();

    output::handle_output(
        &IntentResponse {
            schema_version: SCHEMA_VERSION,
            ok,
            mismatches: mismatches.clone(),
        },
        flags,
    )?;

    if !ok {
        return Err(KiError::Validation);
    }
    Ok(())
}
