use crate::lsclient::{self, LanguageServerRef, ResponseCallable, ResponseCallback};
use lsp_types::{
    ClientCapabilities, CompletionClientCapabilities, CompletionItemCapability,
    CompletionItemCapabilityResolveSupport, CompletionItemKind, CompletionItemKindCapability,
    CompletionItemTag, DynamicRegistrationClientCapabilities, GeneralClientCapabilities,
    InitializeParams, MarkupKind, NotebookDocumentClientCapabilities,
    TextDocumentClientCapabilities, TextDocumentSyncClientCapabilities,
    WorkspaceClientCapabilities,
};
use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    types::{PyDict, PyFunction},
};
use pythonize::pythonize;
use serde::Serialize;
use serde_json::Value;
use std::{
    process::Stdio,
    sync::{Arc, RwLock},
};
use tokio::{process::Command, sync::futures, try_join};

#[pyclass]
pub struct LsClient {
    client: lsclient::LanguageServerRef,
}

#[pymethods]
impl LsClient {
    #[new]
    pub fn new() -> Self {
        Self {
            client: LanguageServerRef::new(),
        }
    }

    pub async fn start(&self, command: String, args: Vec<String>) -> PyResult<()> {
        self.client.start_language_server(&command, args).await;
        Ok(())
    }
    pub async fn send_request<'py>(&self, method: String, params: Py<PyDict>, cb: Py<PyFunction>) {
        self.client
            .send_request(&method, &Value::Null, move |result| {
                pyo3::Python::with_gil(|py| {
                    let is_ok = result.is_ok();
                    let value = match result {
                        Ok(v) => pythonize(py, &v),
                        Err(v) => pythonize(py, &v),
                    }
                    .unwrap();

                    (cb).call1(py, (is_ok, value))
                });
            });
    }
    pub async fn initialize<'py>(&self) {
        let request = InitializeParams {
            process_id: Some(std::process::id()),
            root_path: None,
            root_uri: self.root_uri.clone(),
            initialization_options: None,
            capabilities: ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    configuration: Some(true),
                    did_change_configuration: Some(DynamicRegistrationClientCapabilities {
                        dynamic_registration: Some(true),
                    }),

                    ..Default::default()
                }),
                text_document: Some(TextDocumentClientCapabilities {
                    synchronization: Some(TextDocumentSyncClientCapabilities {
                        dynamic_registration: Some(true),
                        will_save: Some(false),
                        will_save_wait_until: Some(false),
                        did_save: Some(true),
                    }),
                    completion: Some(CompletionClientCapabilities {
                        dynamic_registration: Some(true),
                        completion_item: Some(CompletionItemCapability {
                            snippet_support: Some(true),
                            commit_characters_support: Some(true),
                            documentation_format: Some(vec![
                                MarkupKind::Markdown,
                                MarkupKind::PlainText,
                            ]),
                            deprecated_support: Some(true),
                            preselect_support: Some(true),
                            tag_support: Some(lsp_types::TagSupport {
                                value_set: vec![CompletionItemTag::DEPRECATED],
                            }),
                            insert_replace_support: Some(true),
                            resolve_support: Some(CompletionItemCapabilityResolveSupport {
                                properties: vec![
                                    "documentation".to_string(),
                                    "detail".to_string(),
                                    "additionalTextEdits".to_string(),
                                ],
                            }),
                            ..Default::default()
                        }),
                        completion_item_kind: Some(CompletionItemKindCapability {
                            value_set: Some(vec![
                                CompletionItemKind::TEXT,
                                CompletionItemKind::METHOD,
                                CompletionItemKind::FUNCTION,
                                CompletionItemKind::CONSTRUCTOR,
                                CompletionItemKind::FIELD,
                                CompletionItemKind::VARIABLE,
                                CompletionItemKind::CLASS,
                                CompletionItemKind::INTERFACE,
                                CompletionItemKind::MODULE,
                                CompletionItemKind::PROPERTY,
                                CompletionItemKind::UNIT,
                                CompletionItemKind::VALUE,
                                CompletionItemKind::ENUM,
                                CompletionItemKind::KEYWORD,
                                CompletionItemKind::SNIPPET,
                                CompletionItemKind::COLOR,
                                CompletionItemKind::FILE,
                                CompletionItemKind::REFERENCE,
                                CompletionItemKind::FOLDER,
                                CompletionItemKind::ENUM_MEMBER,
                                CompletionItemKind::CONSTANT,
                                CompletionItemKind::STRUCT,
                                CompletionItemKind::EVENT,
                                CompletionItemKind::OPERATOR,
                                CompletionItemKind::TYPE_PARAMETER,
                            ]),
                        }),
                        context_support: Some(true),
                        ..Default::default()
                    }),
                    publish_diagnostics: Some(PublishDiagnosticsClientCapabilities {
                        related_information: Some(true),
                        tag_support: Some(DiagnosticTagClientCapabilities {
                            value_set: vec![DiagnosticTag::UNNECESSARY, DiagnosticTag::DEPRECATED],
                        }),
                        version_support: Some(true),
                        code_description_support: Some(true),
                        data_support: Some(true),
                    }),
                    ..Default::default()
                }),
                notebook_document: Some(NotebookDocumentClientCapabilities {
                    ..Default::default()
                }),
                ..Default::default()
            },
            trace: Some(TraceValues::Off),
            workspace_folders: self.workspace_folders.clone(),
            client_info: Some(ClientInfo {
                name: "your-editor-name".to_string(),
                version: Some("1.0.0".to_string()),
            }),
            locale: None,
        };
    }
}
