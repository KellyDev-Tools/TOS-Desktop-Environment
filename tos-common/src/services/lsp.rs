use lsp_types::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use url::Url;

use crate::TosState;
use lsp_types::notification::Notification;

pub struct LspClient {
    pub language: String,
    pub tx: crossbeam_channel::Sender<String>,      // Send IPC JSON-RPC requests
    pub diagnostics_rx: crossbeam_channel::Receiver<(String, Vec<Diagnostic>)>,
}

pub struct LspService {
    clients: Arc<Mutex<HashMap<String, Arc<LspClient>>>>, // language -> Client
    state: Arc<Mutex<Option<Arc<Mutex<TosState>>>>>,
    module_manager: Arc<Mutex<Option<Arc<crate::brain::module_manager::ModuleManager>>>>,
}

impl Default for LspService {
    fn default() -> Self {
        Self::new()
    }
}

impl LspService {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(Mutex::new(None)),
            module_manager: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_state(&self, state: Arc<Mutex<TosState>>) {
        *self.state.lock().unwrap() = Some(state);
    }

    pub fn set_module_manager(&self, manager: Arc<crate::brain::module_manager::ModuleManager>) {
        *self.module_manager.lock().unwrap() = Some(manager);
    }

    pub fn start_client(&self, language: &str, cwd: PathBuf) -> Option<Arc<LspClient>> {
        let mut cmd_name = match language {
            "rust" => "rust-analyzer".to_string(),
            "typescript" | "javascript" | "typescriptreact" => "typescript-language-server".to_string(),
            "python" => "pyright-langserver".to_string(),
            _ => "".to_string(),
        };

        let mut args = vec!["--stdio".to_string()];

        // Check for Language Modules (§1.12)
        if cmd_name.is_empty() {
            let mm_lock = self.module_manager.lock().unwrap();
            if let Some(mm) = mm_lock.as_ref() {
                for manifest in mm.list_language_modules() {
                    if let Some(extensions) = &manifest.file_extensions {
                        if extensions.iter().any(|e| e.trim_start_matches('.') == language) || manifest.id == language {
                            if let Some(lsp_config) = &manifest.lsp {
                                cmd_name = lsp_config.command.clone();
                                args = lsp_config.args.clone();
                                break;
                            }
                        }
                    }
                }
            }
        }

        if cmd_name.is_empty() {
            return None;
        }

        // Try binding locally
        let mut child = match Command::new(&cmd_name)
            .args(&args)
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                if language == "typescript" || language == "javascript" {
                    // Try fallback
                    if let Ok(c) = Command::new("npx")
                        .arg("typescript-language-server")
                        .arg("--stdio")
                        .current_dir(&cwd)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::null())
                        .spawn()
                    {
                        c
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
        };

        tracing::info!("[LSP] Spawning {} for language {}", cmd_name, language);

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut reader = BufReader::new(stdout);

        let (tx, rcv) = crossbeam_channel::unbounded::<String>();
        let (_diag_tx, diag_rx) = crossbeam_channel::unbounded();

        let lsp_client = Arc::new(LspClient {
            language: language.to_string(),
            tx,
            diagnostics_rx: diag_rx,
        });

        // Writer Task
        tokio::spawn(async move {
            while let Ok(msg) = rcv.recv() {
                let rpc = format!("Content-Length: {}\r\n\r\n{}", msg.len(), msg);
                if stdin.write_all(rpc.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        // Reader Task
        let state_ref = self.state.clone();
        tokio::spawn(async move {
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                if line.starts_with("Content-Length: ") {
                    let len_str = line["Content-Length: ".len()..].trim();
                    if let Ok(content_length) = len_str.parse::<usize>() {
                        // Skip the empty line
                        reader.read_line(&mut line).await.ok();
                        
                        let mut buf = vec![0u8; content_length];
                        if reader.read_exact(&mut buf).await.is_ok() {
                            if let Ok(msg) = String::from_utf8(buf) {
                                if let Ok(json_rpc) = serde_json::from_str::<Value>(&msg) {
                                    // Intercept PublishDiagnostics
                                    if let Some(method) = json_rpc.get("method").and_then(|m| m.as_str()) {
                                        if method == lsp_types::notification::PublishDiagnostics::METHOD {
                                            if let Ok(params) = serde_json::from_value::<lsp_types::PublishDiagnosticsParams>(
                                                json_rpc.get("params").unwrap().clone(),
                                            ) {
                                                let state_lock = state_ref.lock().unwrap();
                                                if let Some(st) = state_lock.as_ref() {
                                                    let mut s = st.lock().unwrap();
                                                let uri_str = params.uri.to_string();
                                                let url = Url::parse(&uri_str).unwrap();
                                                let file_path_str = url.to_file_path().unwrap().to_string_lossy().to_string();
                                                
                                                // Map to EditorAnnotations
                                                let mut new_anns = vec![];
                                                for diag in params.diagnostics {
                                                    let severity = match diag.severity {
                                                        Some(DiagnosticSeverity::ERROR) => "error",
                                                        Some(DiagnosticSeverity::WARNING) => "warning",
                                                        _ => "info",
                                                    };
                                                    new_anns.push(crate::state::EditorAnnotation {
                                                        line: diag.range.start.line as usize,
                                                        severity: severity.to_string(),
                                                        message: diag.message,
                                                    });
                                                }

                                                // Update in State
                                                let mut any_updated = false;
                                                let idx = s.active_sector_index;
                                                if let Some(sector) = s.sectors.get_mut(idx) {
                                                    for hub in sector.hubs.iter_mut() {
                                                        if let Some(layout) = hub.split_layout.as_mut() {
                                                            fn update_pane(node: &mut crate::SplitNode, path: &str, anns: Vec<crate::state::EditorAnnotation>) -> bool {
                                                                match node {
                                                                    crate::SplitNode::Leaf(ref mut p) => {
                                                                        if let crate::PaneContent::Editor(ref mut ed) = p.content {
                                                                            if ed.file_path.to_string_lossy() == path {
                                                                                ed.annotations = anns;
                                                                                return true;
                                                                            }
                                                                        }
                                                                        false
                                                                    }
                                                                    crate::SplitNode::Container { ref mut children, .. } => {
                                                                        children.iter_mut().any(|c| update_pane(c, path, anns.clone()))
                                                                    }
                                                                }
                                                            }
                                                            if update_pane(layout, &file_path_str, new_anns.clone()) {
                                                                any_updated = true;
                                                            }
                                                        }
                                                    }
                                                }
                                                if any_updated {
                                                    s.version += 1;
                                                }
                                                } // end st
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                line.clear();
            }
        });

        // Initialize RPC
        let root_uri_parsed: lsp_types::Uri = Url::from_file_path(&cwd).unwrap().to_string().parse().unwrap();
        #[allow(deprecated)]
        let init_params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: None,
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: root_uri_parsed,
                name: "workspace".to_string(),
            }]),
            capabilities: ClientCapabilities::default(),
            ..Default::default()
        };

        if let Ok(json_val) = serde_json::to_value(&init_params) {
            let rpc = json!({
                "jsonrpc": "2.0",
                "id": 0,
                "method": "initialize",
                "params": json_val
            });
            let _ = lsp_client.tx.send(serde_json::to_string(&rpc).unwrap());
        }

        self.clients.lock().unwrap().insert(language.to_string(), lsp_client.clone());
        Some(lsp_client)
    }

    pub fn did_open(&self, language: &str, file_path: &std::path::Path, content: &str) {
        if let Some(client) = self.clients.lock().unwrap().get(language) {
            let params = DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: Url::from_file_path(file_path).unwrap().to_string().parse().unwrap(),
                    language_id: language.to_string(),
                    version: 1,
                    text: content.to_string(),
                },
            };
            let rpc = json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": serde_json::to_value(&params).unwrap()
            });
            let _ = client.tx.send(serde_json::to_string(&rpc).unwrap());
        }
    }

    pub fn did_change(&self, language: &str, file_path: &std::path::Path, text: &str) {
        if let Some(client) = self.clients.lock().unwrap().get(language) {
            let params = DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri: Url::from_file_path(file_path).unwrap().to_string().parse().unwrap(),
                    version: 2,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    range: None,
                    range_length: None,
                    text: text.to_string(),
                }],
            };
            let rpc = json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didChange",
                "params": serde_json::to_value(&params).unwrap()
            });
            let _ = client.tx.send(serde_json::to_string(&rpc).unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_lsp_message_generation() {
        let (tx, rx) = crossbeam_channel::unbounded();
        let diag_rx = crossbeam_channel::unbounded().1;
        
        let client = Arc::new(LspClient {
            language: "rust".to_string(),
            tx,
            diagnostics_rx: diag_rx,
        });

        let service = LspService::new();
        service.clients.lock().unwrap().insert("rust".to_string(), client);

        let path = Path::new("/tmp/test.rs");
        service.did_open("rust", path, "fn main() {}");

        let msg = rx.recv().unwrap();
        let json: Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(json["method"], "textDocument/didOpen");
        assert_eq!(json["params"]["textDocument"]["text"], "fn main() {}");

        service.did_change("rust", path, "fn main() { println!(); }");
        let msg2 = rx.recv().unwrap();
        let json2: Value = serde_json::from_str(&msg2).unwrap();
        assert_eq!(json2["method"], "textDocument/didChange");
        assert_eq!(json2["params"]["contentChanges"][0]["text"], "fn main() { println!(); }");
    }
}
