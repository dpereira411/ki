use kicad_ipc_rs::KiCadError;

pub fn render_kicad_error(err: &KiCadError) -> Vec<String> {
    let mut lines = vec![format!("error: {err}")];

    match err {
        KiCadError::SocketUnavailable { .. } => {
            lines.push(
                "hint: launch KiCad and ensure its IPC API socket is available before rerunning this command."
                    .to_string(),
            );
        }
        KiCadError::ApiStatus { code, message } if code == "AS_UNHANDLED" => {
            if message.contains("kiapi.common.commands.GetOpenDocuments") {
                lines.push(
                    "hint: open the target editor first before using `ki refresh` (for `--frame pcb`, open pcbnew; for `--frame schematic`, open eeschema)."
                        .to_string(),
                );
            } else if message.contains("kiapi.common.commands.RevertDocument") {
                lines.push(
                    "hint: this KiCad build does not expose `RevertDocument` for that frame, so `ki refresh` cannot reload it through IPC."
                        .to_string(),
                );
                lines.push(
                    "hint: as of KiCad 10, IPC refresh/reload support is still effectively PCB-focused; non-PCB editors may not expose a reload path."
                        .to_string(),
                );
            } else if message.contains("kiapi.common.commands.RefreshEditor") {
                lines.push(
                    "hint: this KiCad build does not expose `RefreshEditor`; `ki refresh` will only work where document reload is available."
                        .to_string(),
                );
            }
        }
        KiCadError::ApiStatus { code, message }
            if code == "AS_BAD_REQUEST"
                && message.contains("requested document")
                && message.contains("is not open") =>
        {
            lines.push(
                "hint: KiCad rejected the target non-PCB document for IPC reload, even though the editor may be open."
                    .to_string(),
            );
            lines.push(
                "hint: as of KiCad 10, IPC refresh/reload support is still effectively PCB-focused; non-PCB editors may not expose a reliable reload path."
                    .to_string(),
            );
        }
        _ => {}
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_socket_hint() {
        let lines = render_kicad_error(&KiCadError::SocketUnavailable {
            socket_uri: "ipc:///tmp/kicad.sock".to_string(),
        });

        assert!(lines.iter().any(|line| line.contains("IPC API socket")));
    }

    #[test]
    fn renders_refresh_unhandled_hint() {
        let lines = render_kicad_error(&KiCadError::ApiStatus {
            code: "AS_UNHANDLED".to_string(),
            message: "no handler available for request of type kiapi.common.commands.RefreshEditor"
                .to_string(),
        });

        assert!(lines.iter().any(|line| line.contains("RefreshEditor")));
    }

    #[test]
    fn renders_open_editor_hint_for_get_open_documents() {
        let lines = render_kicad_error(&KiCadError::ApiStatus {
            code: "AS_UNHANDLED".to_string(),
            message:
                "no handler available for request of type kiapi.common.commands.GetOpenDocuments"
                    .to_string(),
        });

        assert!(lines
            .iter()
            .any(|line| line.contains("open the target editor first")));
    }

    #[test]
    fn renders_non_pcb_support_hint_for_revert_document() {
        let lines = render_kicad_error(&KiCadError::ApiStatus {
            code: "AS_UNHANDLED".to_string(),
            message:
                "no handler available for request of type kiapi.common.commands.RevertDocument"
                    .to_string(),
        });

        assert!(lines.iter().any(|line| line.contains("KiCad 10")));
    }

    #[test]
    fn renders_non_pcb_bad_request_hint() {
        let lines = render_kicad_error(&KiCadError::ApiStatus {
            code: "AS_BAD_REQUEST".to_string(),
            message: "the requested document FlipFlop.kicad_sch is not open".to_string(),
        });

        assert!(lines.iter().any(|line| line.contains("non-PCB document")));
        assert!(lines.iter().any(|line| line.contains("KiCad 10")));
    }
}
