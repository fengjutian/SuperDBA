//! Incremental parsers for provider streaming responses.
//!
//! The HTTP layer feeds lines here and forwards text deltas to the caller; the
//! accumulated state produces the same [`AssistantReply`] the non-streaming
//! parsers build, so the agent loop does not branch on transport shape.

use super::provider::{AssistantReply, ProviderToolCall};
use serde_json::Value;

/// Splits complete lines out of a growing buffer. Returns the lines without
/// their terminators and leaves any partial line in `buffer`.
pub fn take_lines(buffer: &mut String) -> Vec<String> {
    let mut lines = Vec::new();
    while let Some(position) = buffer.find('\n') {
        let line: String = buffer.drain(..=position).collect();
        let line = line.trim_end_matches(['\r', '\n']);
        if !line.is_empty() {
            lines.push(line.to_string());
        }
    }
    lines
}

#[derive(Clone, Debug, Default)]
pub(super) struct ToolCallDraft {
    id: String,
    name: String,
    arguments: String,
}

impl ToolCallDraft {
    fn into_call(self, index: usize) -> Option<ProviderToolCall> {
        if self.name.is_empty() {
            return None;
        }
        let arguments = if self.arguments.trim().is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&self.arguments)
                .unwrap_or_else(|_| serde_json::json!({ "raw": self.arguments }))
        };
        Some(ProviderToolCall {
            id: if self.id.is_empty() {
                format!("call_stream_{index}")
            } else {
                self.id
            },
            name: self.name,
            arguments,
        })
    }
}

/// OpenAI-compatible `chat.completion.chunk` accumulator.
#[derive(Debug, Default)]
pub struct OpenAiStream {
    content: String,
    reasoning: String,
    calls: Vec<ToolCallDraft>,
}

impl OpenAiStream {
    /// Applies one chunk and returns the text delta, when the chunk has one.
    pub fn apply(&mut self, chunk: &Value) -> Option<String> {
        if let Some(reasoning) = chunk
            .pointer("/choices/0/delta/reasoning_content")
            .and_then(Value::as_str)
        {
            self.reasoning.push_str(reasoning);
        }
        if let Some(array) = chunk
            .pointer("/choices/0/delta/tool_calls")
            .and_then(Value::as_array)
        {
            for entry in array {
                let index = entry.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                while self.calls.len() <= index {
                    self.calls.push(ToolCallDraft::default());
                }
                let draft = &mut self.calls[index];
                if let Some(id) = entry.get("id").and_then(Value::as_str) {
                    if !id.is_empty() {
                        draft.id = id.to_string();
                    }
                }
                if let Some(name) = entry.pointer("/function/name").and_then(Value::as_str) {
                    draft.name.push_str(name);
                }
                if let Some(arguments) =
                    entry.pointer("/function/arguments").and_then(Value::as_str)
                {
                    draft.arguments.push_str(arguments);
                }
            }
        }
        let text = chunk
            .pointer("/choices/0/delta/content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if text.is_empty() {
            return None;
        }
        self.content.push_str(&text);
        Some(text)
    }

    pub fn finish(self) -> AssistantReply {
        AssistantReply {
            content: self.content,
            reasoning: (!self.reasoning.is_empty()).then_some(self.reasoning),
            tool_calls: self
                .calls
                .into_iter()
                .enumerate()
                .filter_map(|(index, draft)| draft.into_call(index))
                .collect(),
            notice: None,
        }
    }
}

/// Anthropic Messages stream accumulator.
#[derive(Debug, Default)]
pub struct AnthropicStream {
    content: String,
    calls: Vec<ToolCallDraft>,
    current: Option<usize>,
}

impl AnthropicStream {
    pub fn apply(&mut self, event: &Value) -> Option<String> {
        match event.get("type").and_then(Value::as_str) {
            Some("content_block_start") => {
                let index = event.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                let block = event.get("content_block");
                let kind = block
                    .and_then(|block| block.get("type"))
                    .and_then(Value::as_str);
                if kind == Some("tool_use") {
                    let mut draft = ToolCallDraft::default();
                    if let Some(id) = block
                        .and_then(|block| block.get("id"))
                        .and_then(Value::as_str)
                    {
                        draft.id = id.to_string();
                    }
                    if let Some(name) = block
                        .and_then(|block| block.get("name"))
                        .and_then(Value::as_str)
                    {
                        draft.name = name.to_string();
                    }
                    while self.calls.len() <= index {
                        self.calls.push(ToolCallDraft::default());
                    }
                    self.calls[index] = draft;
                    self.current = Some(index);
                } else {
                    self.current = None;
                }
                None
            }
            Some("content_block_delta") => {
                let delta = event.get("delta");
                match delta
                    .and_then(|delta| delta.get("type"))
                    .and_then(Value::as_str)
                {
                    Some("text_delta") => {
                        let text = delta
                            .and_then(|delta| delta.get("text"))
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        if text.is_empty() {
                            return None;
                        }
                        self.content.push_str(&text);
                        Some(text)
                    }
                    Some("input_json_delta") => {
                        if let (Some(index), Some(partial)) = (
                            self.current,
                            delta
                                .and_then(|delta| delta.get("partial_json"))
                                .and_then(Value::as_str),
                        ) {
                            if let Some(draft) = self.calls.get_mut(index) {
                                draft.arguments.push_str(partial);
                            }
                        }
                        None
                    }
                    _ => None,
                }
            }
            Some("content_block_stop") => {
                self.current = None;
                None
            }
            _ => None,
        }
    }

    pub fn finish(self) -> AssistantReply {
        AssistantReply {
            content: self.content,
            reasoning: None,
            tool_calls: self
                .calls
                .into_iter()
                .enumerate()
                .filter_map(|(index, draft)| draft.into_call(index))
                .collect(),
            notice: None,
        }
    }
}

/// Ollama streams newline-delimited JSON messages.
#[derive(Debug, Default)]
pub struct OllamaStream {
    content: String,
    calls: Vec<ProviderToolCall>,
}

impl OllamaStream {
    pub fn apply(&mut self, message: &Value) -> Option<String> {
        if let Some(array) = message.get("tool_calls").and_then(Value::as_array) {
            for entry in array {
                let name = entry
                    .pointer("/function/name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                if name.is_empty() {
                    continue;
                }
                let arguments = entry
                    .pointer("/function/arguments")
                    .cloned()
                    .unwrap_or(Value::Null);
                self.calls.push(ProviderToolCall {
                    id: format!("call_stream_{}", self.calls.len()),
                    name,
                    arguments: if arguments.is_object() {
                        arguments
                    } else {
                        super::provider::parse_arguments(&arguments)
                    },
                });
            }
        }
        let text = message
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if text.is_empty() {
            return None;
        }
        self.content.push_str(&text);
        Some(text)
    }

    pub fn finish(self) -> AssistantReply {
        AssistantReply {
            content: self.content,
            reasoning: None,
            tool_calls: self.calls,
            notice: None,
        }
    }
}
