use lsp_types::notification::{DidOpenTextDocument, Exit, Initialized, Notification};
use lsp_types::request::{Completion, Initialize, Request, Shutdown};
use lsp_types::{
    lsp_request, ClientCapabilities, CompletionClientCapabilities, CompletionParams,
    CompletionResponse, DidOpenTextDocumentParams, HoverClientCapabilities, InitializeParams,
    PartialResultParams, Position, TextDocumentClientCapabilities, TextDocumentIdentifier,
    TextDocumentItem, TextDocumentPositionParams, TextDocumentSyncClientCapabilities, Uri,
    WorkDoneProgressParams, WorkspaceFolder,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env::var;
use std::error::Error;
use std::process::Stdio;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::mpsc::{self, UnboundedSender};

#[derive(Debug, Serialize, Deserialize)]
pub struct LspMessage {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

pub struct LspClient {
    process: Option<Child>,
    request_id: u32,
    sender: Option<mpsc::UnboundedSender<String>>,
    reciever: Option<mpsc::UnboundedReceiver<String>>,
    request_types: Arc<RwLock<HashMap<u32, String>>>,
    response_reciever: Option<mpsc::UnboundedReceiver<LspMessage>>,
    response_sender: Option<mpsc::UnboundedSender<LspMessage>>,
}
#[derive(Debug)]
struct ResponseHeader {
    content_length: Option<usize>,
    content_type: Option<String>,
}

async fn parse_header(
    reader: &mut BufReader<ChildStdout>,
) -> Result<ResponseHeader, Box<dyn std::error::Error>> {
    let mut line = String::new();
    let mut content_length: Option<usize> = None;
    let mut content_type: Option<String> = None;
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                continue;
            }
        }

        let line = line.trim();
        if line.is_empty() {
            break; // End of headers
        }

        if let Some((key, value)) = line.split_once(": ") {
            match key {
                "Content-Length" => {
                    content_length = match value.parse::<usize>() {
                        Ok(cl) => Some(cl),
                        Err(e) => {
                            return Err(
                                std::io::Error::new(std::io::ErrorKind::InvalidData, e).into()
                            );
                        }
                    }
                }
                // TODO: check if type is valid
                "Content-Type" => content_type = Some(value.into()),
                _ => {}
            };
        }
    }
    return Ok(ResponseHeader {
        content_length,
        content_type,
    });
}

async fn parse_content(
    reader: &mut BufReader<ChildStdout>,
    content_length: usize,
) -> Result<LspMessage, Box<dyn std::error::Error>> {
    let mut content_buffer = vec![0u8; content_length];

    reader.read_exact(&mut content_buffer).await?;
    let content = String::from_utf8(content_buffer)?;

    // Parse JSON and Return
    serde_json::from_str::<LspMessage>(&content).map_err(|err| err.into())
}

impl LspClient {
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let (response_sender, response_reciever) = mpsc::unbounded_channel::<LspMessage>();
        Self {
            process: None,
            request_id: 0u32,
            sender: Some(tx),
            reciever: Some(rx),
            request_types: Arc::new(RwLock::new(HashMap::new())),
            response_reciever: Some(response_reciever),
            response_sender: Some(response_sender),
        }
    }

    pub async fn start(
        &mut self,
        command: &str,
        args: &[&str],
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting language server: {} {:?}", command, args);

        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        // Channel for sending messages
        // todo handle error
        let mut rx = self.reciever.take().unwrap();
        // Task to write messages to server
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stdin.write_all(message.as_bytes()).await {
                    eprintln!("Failed to write to server: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    eprintln!("Failed to flush: {}", e);
                    break;
                }
            }
        });

        // Task to handle stderr
        tokio::spawn(async move {
            let mut stderr_reader = BufReader::new(stderr);
            let mut line = String::new();
            while let Ok(bytes_read) = stderr_reader.read_line(&mut line).await {
                if bytes_read == 0 {
                    break;
                }
                eprintln!("LSP stderr: {}", line.trim());
                line.clear();
            }
        });

        // Task to read responses from server
        let response_sender = self.response_sender.take().unwrap();

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            loop {
                // Read headers
                let response = match parse_header(&mut reader).await {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Error parsing response header: {e}");
                        continue;
                    }
                };
                let Some(content_length) = response.content_length else {
                    continue;
                };

                let message = match parse_content(&mut reader, content_length).await {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Error parsing response body: {e}");
                        continue;
                    }
                };

                if let Err(e) = response_sender.send(message) {
                    eprintln!("{e}");
                }
            }
        });

        self.process = Some(child);
        println!("Language server started");
        Ok(())
    }
    fn handle_completion(value: Value) -> Result<(), Box<dyn Error>> {
        let response = serde_json::from_value::<CompletionResponse>(value)?;
        println!("{response:#?}");
        Ok(())
    }

    pub async fn handle_response(&mut self) {
        let mut types = self.request_types.clone();
        let Some(mut rx) = self.response_reciever.take() else {
            return;
        };
        tokio::spawn(async move {
            let handle_response =
                |response: LspMessage, response_type: String| match response_type.as_str() {
                    Completion::METHOD => {
                        if let Some(result) = response.result {
                            Self::handle_completion(result);
                        }
                    }
                    &_ => {}
                };
            let handle_notification = |message: LspMessage| {};

            while let Some(response) = rx.recv().await {
                let response_type = response
                    .id
                    .and_then(|id| types.try_write().ok().and_then(|mut t| t.remove(&id)));

                match response_type {
                    Some(response_type) => handle_response(response, response_type),
                    None => handle_notification(response),
                }
            }
        });
    }

    fn create_message(&mut self, method: &str, params: Value, is_request: bool) -> (String, u32) {
        let mut message = LspMessage {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some(method.to_string()),
            params: Some(params),
            result: None,
            error: None,
        };

        let current_id = if is_request {
            self.request_id += 1;
            message.id = Some(self.request_id);

            // Store the request type
            if let Ok(mut types) = self.request_types.try_write() {
                types.insert(self.request_id, method.to_string());
            }

            self.request_id
        } else {
            0 // Notifications don't have IDs
        };

        let content = serde_json::to_string(&message).unwrap();
        (
            format!("Content-Length: {}\r\n\r\n{}", content.len(), content),
            current_id,
        )
    }

    pub async fn send_request<T: Request>(
        &mut self,
        params: Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (message, _) = self.create_message(T::METHOD, params, true);
        // println!("Sending request:\n{}", message);

        if let Some(sender) = &self.sender {
            sender.send(message)?;
        } else {
            return Err("LSP client not started".into());
        }

        Ok(())
    }

    pub async fn send_notification<T: Notification>(
        &mut self,
        params: Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (message, _) = self.create_message(T::METHOD, params, false);
        println!("Sending notification:\n{}", message);

        if let Some(sender) = &self.sender {
            sender.send(message)?;
        } else {
            return Err("LSP client not started".into());
        }

        Ok(())
    }

    pub async fn initialize(&mut self, root_uri: &str) -> Result<(), Box<dyn std::error::Error>> {
        let uri = Uri::from_str(root_uri).map_err(|e| format!("Invalid URI: {}", e))?;

        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: Some(uri.clone()),
            workspace_folders: Some(vec![WorkspaceFolder {
                uri,
                name: root_uri.to_string(),
            }]),
            capabilities: ClientCapabilities {
                workspace: None,
                text_document: Some(TextDocumentClientCapabilities {
                    synchronization: Some(TextDocumentSyncClientCapabilities {
                        dynamic_registration: Some(false),
                        will_save: Some(true),
                        did_save: Some(true),
                        ..Default::default()
                    }),
                    completion: Some(CompletionClientCapabilities {
                        dynamic_registration: Some(false),
                        ..Default::default()
                    }),
                    hover: Some(HoverClientCapabilities {
                        dynamic_registration: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        let params = serde_json::to_value(params)
            .map_err(|e| format!("Failed to serialize params: {}", e))?;
        self.send_request::<Initialize>(params).await?;
        Ok(())
    }

    pub async fn initialized(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_notification::<Initialized>(serde_json::json!({}))
            .await
    }

    pub async fn did_open(
        &mut self,
        uri: &str,
        language_id: &str,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let uri = Uri::from_str(uri)?;
        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                language_id: language_id.into(),
                text: text.into(),
                uri,
                version: 1, // Start with version 1, not 0
            },
        };

        let params = serde_json::to_value(params)?;
        self.send_notification::<DidOpenTextDocument>(params).await
    }

    pub async fn completion(
        &mut self,
        uri: &str,
        line: u32,
        character: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let uri = Uri::from_str(uri)?;
        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                position: Position { line, character },
                text_document: TextDocumentIdentifier { uri },
            },
            context: None,
            partial_result_params: PartialResultParams::default(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };

        let params = serde_json::to_value(params)?;
        self.send_request::<lsp_request!("textDocument/completion")>(params)
            .await
    }

    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Shutting down LSP client...");

        self.send_request::<Shutdown>(serde_json::json!({})).await?;

        // Wait a bit for shutdown response
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        self.send_notification::<Exit>(serde_json::json!({}))
            .await?;

        if let Some(mut process) = self.process.take() {
            // Give the process a moment to exit gracefully
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let _ = process.kill().await;
        }

        Ok(())
    }
}
