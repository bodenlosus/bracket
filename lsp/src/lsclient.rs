use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::errors::ServerError;
use crate::parse;
use crate::utils::LspMessage;

// Class for Communication, does not handle any types
pub struct LanguageServer {
    ls_stdin: Option<ChildStdin>,
    next_id: usize,
    pending: HashMap<usize, ResponseCallback>,
    subscriptions: HashMap<String, NotificationCallback>,
}

pub trait NotificationCallable: 'static + Send + FnMut(Option<Value>) {}

pub type ResponseCallback = Box<dyn 'static + Send + FnOnce(Result<Value, Value>)>;
pub type NotificationCallback = Box<dyn NotificationCallable>;

// impl Display for ServerError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ServerError::NotificationCallbackNotFound(id)
//         }
//         write!(
//             f,
//             "No Callback Found for Message with id {} as a response to request type {:?}",
//             self.message_id, self.method
//         )
//     }
// }

impl LanguageServer {
    fn new() -> Self {
        Self {
            ls_stdin: None,
            next_id: 1,
            pending: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }
    fn set_stdin(&mut self, stdin: ChildStdin) {
        self.ls_stdin = Some(stdin)
    }
    async fn send_request(
        &mut self,
        method: &str,
        params: &Value,
        completion: ResponseCallback,
    ) -> Result<(), Box<dyn Error>> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": self.next_id,
            "method": method,
            "params": params
        });

        self.pending.insert(self.next_id, completion);
        self.next_id += 1;
        self.send_rpc(&request).await?;
        Ok(())
    }
    async fn subscribe_to_notification<CB: NotificationCallable>(
        &mut self,
        method: &str,
        cb: Box<CB>,
    ) -> () {
        self.subscriptions.insert(method.to_string(), cb);
    }

    async fn send_notification(
        &mut self,
        method: &str,
        params: &Value,
    ) -> Result<(), Box<dyn Error>> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.send_rpc(&notification).await?;
        Ok(())
    }
    async fn send_rpc(&mut self, rpc: &Value) -> Result<(), Box<dyn Error>> {
        let msg = parse::prepare_lsp_json(rpc)?;
        let stdin = self
            .ls_stdin
            .as_mut()
            .expect("No Stdin set for Language Server");
        stdin.write_all(msg.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }
    async fn handle_response_succesful(
        &mut self,
        id: usize,
        result: Value,
    ) -> Result<(), Box<dyn Error>> {
        let cb = self
            .pending
            .remove(&id)
            .ok_or(ServerError::ResponseCallbackNotFound(id))?;
        (cb)(Ok(result));
        Ok(())
    }
    async fn handle_notification(
        &mut self,
        method: String,
        params: Option<Value>,
    ) -> Result<(), Box<dyn Error>> {
        let cb = self.subscriptions.get_mut(&method);
        if let Some(cb) = cb {
            (cb)(params)
        }
        Ok(())
    }
    async fn handle_response_error(
        &mut self,
        id: usize,
        error: Value,
    ) -> Result<(), Box<dyn Error>> {
        let cb = self
            .pending
            .remove(&id)
            .ok_or(ServerError::ResponseCallbackNotFound(id))?;
        (cb)(Err(error));
        Ok(())
    }
    async fn handle_rpc(&mut self, message: LspMessage) {
        match message {
            LspMessage {
                id: Some(id),
                result: Some(result),
                error: None,
                method: None,
                params: None,
                ..
            } => self.handle_response_succesful(id, result).await,
            LspMessage {
                id: None,
                method: Some(method),
                params: params,
                result: None,
                error: None,
                ..
            } => self.handle_notification(method, params).await,
            LspMessage {
                id: Some(id),
                method: None,
                result: None,
                error: Some(error),
                params: None,
                ..
            } => self.handle_response_error(id, error).await,
            LspMessage { .. } => Ok(()),
        };
    }
}

pub struct LanguageServerRef(Arc<Mutex<LanguageServer>>);
impl Clone for LanguageServerRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl LanguageServerRef {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(LanguageServer::new())))
    }
    fn set_stdin(&self, stdin: ChildStdin) -> Result<(), Box<dyn Error>> {
        let mut guard = self
            .0
            .try_lock()
            .map_err(|_| "Could not acquire lock on ls guard")?;

        guard.set_stdin(stdin);
        Ok(())
    }
    pub fn send_request<CB>(
        &self,
        method: &str,
        params: &Value,
        cb: CB,
    ) -> Result<(), Box<dyn Error>>
    where
        CB: 'static + Send + FnOnce(Result<Value, Value>),
    {
        let mut guard = self
            .0
            .try_lock()
            .map_err(|_| "Could not acquire lock on ls guard")?;

        guard.send_request(method, params, Box::new(cb));
        Ok(())
    }
    pub fn send_notification(&self, method: &str, params: &Value) -> Result<(), Box<dyn Error>> {
        let mut guard = self
            .0
            .try_lock()
            .map_err(|_| "Could not acquire lock on ls guard")?;

        guard.send_notification(method, params);
        Ok(())
    }
    pub fn subscribe_to_notification<CB: NotificationCallable>(
        &self,
        method: &str,
        cb: CB,
    ) -> Result<(), Box<dyn Error>> {
        let mut guard = self
            .0
            .try_lock()
            .map_err(|_| "Could not acquire lock on ls guard")?;

        guard.subscribe_to_notification(method, Box::new(cb));
        Ok(())
    }
    pub fn handle_message(&self, message: LspMessage) -> Result<(), Box<dyn Error>> {
        let mut guard = self
            .0
            .try_lock()
            .map_err(|_| "Could not acquire lock on ls guard")?;

        guard.handle_rpc(message);
        Ok(())
    }

    pub async fn start_language_server<S: AsRef<OsStr>, I: IntoIterator<Item = S>>(
        &self,
        command: &str,
        args: I,
    ) {
        let mut child = Command::new(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(args)
            .spawn()
            .expect(format!("Failed to run {} with args", command).as_str());
        let child_stdin = child.stdin.take().unwrap();
        let child_stdout = child.stdout.take().unwrap();
        self.set_stdin(child_stdin).expect("Could not set stdin");
        {
            let lang_server = self.clone();
            tokio::task::spawn(async move {
                let mut reader = tokio::io::BufReader::new(child_stdout);
                loop {
                    let msg = match parse::read_message(&mut reader).await {
                        Ok(msg) => msg,
                        Err(e) => {
                            eprintln!("Error parsing Message {}", e);
                            continue;
                        }
                    };

                    if let Err(e) = lang_server.handle_message(msg) {
                        eprintln!("Error handling message: {e}")
                    }
                }
            });
        }
    }
}
