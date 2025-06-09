use serde_json::Value;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, BufReader},
    process::ChildStdout,
};

use crate::{errors::ParseError, utils::LspMessage};

enum LspHeaderLine {
    ContentType(String),
    ContentLength(usize),
}

pub async fn read_message(reader: &mut BufReader<ChildStdout>) -> Result<LspMessage, ParseError> {
    let mut buffer = String::new();
    let mut content_length = None::<usize>;

    loop {
        buffer.clear();
        reader.read_line(&mut buffer).await?;

        let buffer = buffer.trim();

        if buffer.is_empty() {
            break;
        }

        let lsp_header_line = parse_header(&buffer).await;

        match lsp_header_line? {
            LspHeaderLine::ContentLength(l) => content_length = Some(l),
            LspHeaderLine::ContentType(_) => {} // TODO: Allow other then utf8
        }
    }
    let content_length =
        content_length.ok_or(format!("missing content-length section in header {buffer}"))?;

    let mut body_buffer = vec![0u8; content_length];
    reader.read_exact(&mut body_buffer);
    let body = String::from_utf8(body_buffer)?;

    Ok(serde_json::from_str::<LspMessage>(&body)?)
}

const HEADER_CONTENT_LENGTH: &'static str = "content-length";
const HEADER_CONTENT_TYPE: &'static str = "content-type";

async fn parse_header(header: &str) -> Result<LspHeaderLine, ParseError> {
    let (keyword, value) = header
        .split_once(": ")
        .ok_or(ParseError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Malformed header: {header}"),
        )))?;
    let keyword = keyword.trim().to_lowercase();
    match keyword.as_str() {
        HEADER_CONTENT_TYPE => Ok(LspHeaderLine::ContentType(value.to_string())),
        HEADER_CONTENT_LENGTH => Ok(LspHeaderLine::ContentLength(usize::from_str_radix(
            value, 10,
        )?)),
        &_ => Err(ParseError::Unknown(format!("Invalid header: {keyword}"))),
    }
}
pub fn prepare_lsp_json(msg: &Value) -> Result<String, serde_json::error::Error> {
    let request = serde_json::to_string(&msg)?;
    Ok(format!(
        "Content-Length: {}\r\n\r\n{}",
        request.len(),
        request
    ))
}
