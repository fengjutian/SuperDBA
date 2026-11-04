//! Provider adapters for tool calling.
//!
//! Two wire formats are supported: the OpenAI chat-completions tool format
//! (OpenAI, DeepSeek, MiniMax, OpenRouter, Requesty, custom OpenAI-compatible
//! endpoints, and Ollama's native chat API) and the Anthropic Messages tool
//! format. Providers that cannot call tools return a typed error instead of
//! silently ignoring the tool definitions.

use super::tools::ToolSchema;
use super::{ChatMessage, ChatRole};
use crate::ai::{minimax_endpoint_candidates, remember_minimax_endpoint, MiniMaxEndpoint};
use crate::application::ai::get_api_key;
use crate::config::AppConfig;
use crate::credential_cache::CredentialCache;
use crate::runtime::RuntimeContext;
use serde_json::{json, Value};

/// A tool call as returned by a provider, before execution.
#[derive(Clone, Debug, PartialEq)]
pub struct ProviderToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Clone, Debug)]
pub struct AssistantReply {
    pub content: String,
    /// Provider-side reasoning that must be replayed with the assistant turn
    /// (MiniMax interleaved thinking). Other providers ignore it.
    pub reasoning: Option<String>,
    pub tool_calls: Vec<ProviderToolCall>,
    pub notice: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GenerationOptions {
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<u32>,
}

pub(super) fn apply_generation_options(
    body: &mut Value,
    provider: &str,
    options: GenerationOptions,
) {
    if provider == "ollama" {
        let config = body
            .as_object_mut()
            .expect("Ollama request body must be an object")
            .entry("options")
            .or_insert_with(|| json!({}));
        if let Some(temperature) = options.temperature {
            config["temperature"] = json!(temperature);
        }
        if let Some(max_tokens) = options.max_output_tokens {
            config["num_predict"] = json!(max_tokens);
        }
        return;
    }
    if let Some(temperature) = options.temperature {
        body["temperature"] = json!(temperature);
    }
    if let Some(max_tokens) = options.max_output_tokens {
        body["max_tokens"] = json!(max_tokens);
    }
}

const UNSUPPORTED_TOOL_CALLING: &str =
    "This provider does not support chat tools yet. Pick an OpenAI, Anthropic, DeepSeek, MiniMax, OpenRouter, Requesty, custom OpenAI-compatible, or Ollama model in Settings → AI.";

pub fn supports_tool_calling(provider: &str) -> bool {
    matches!(
        provider,
        "openai"
            | "anthropic"
            | "deepseek"
            | "minimax"
            | "openrouter"
            | "requesty"
            | "custom-openai"
            | "ollama"
    )
}

pub async fn complete(
    runtime: &RuntimeContext,
    cache: &CredentialCache,
    config: &AppConfig,
    provider: &str,
    model: &str,
    system: &str,
    messages: &[ChatMessage],
    tools: &[ToolSchema],
) -> Result<AssistantReply, String> {
    if !supports_tool_calling(provider) {
        return Err(UNSUPPORTED_TOOL_CALLING.to_string());
    }

    match provider {
        "anthropic" => {
            let api_key = get_api_key(runtime, cache, provider)?;
            let client = crate::proxy::ai_http_client(provider)?;
            let body = anthropic_body(model, system, messages, tools);
            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|error| format!("Anthropic request failed: {error}"))?;
            let value = decode_response(response, "Anthropic").await?;
            parse_anthropic(&value)
        }
        "minimax" => {
            let api_key = get_api_key(runtime, cache, provider)?;
            let client = crate::proxy::ai_http_client(provider)?;
            let body = minimax_body(model, system, messages, tools);
            minimax_complete(&client, &api_key, &body).await
        }
        "ollama" => {
            let client = crate::proxy::direct_http_client()?;
            let port = config.ai_ollama_port.unwrap_or(11434);
            let body = ollama_body(model, system, messages, tools);
            let response = client
                .post(format!("http://localhost:{port}/api/chat"))
                .json(&body)
                .send()
                .await
                .map_err(|error| format!("Failed to connect to Ollama on port {port}: {error}"))?;
            let value = decode_response(response, "Ollama").await?;
            parse_ollama(&value)
        }
        _ => {
            let api_key = get_api_key(runtime, cache, provider)?;
            let client = crate::proxy::ai_http_client(provider)?;
            let url = openai_endpoint(provider, config)?;
            let body = openai_body(model, system, messages, tools);
            let mut request = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body);
            if !api_key.is_empty() {
                request = request.header("Authorization", format!("Bearer {api_key}"));
            }
            if matches!(provider, "openrouter" | "requesty") {
                request = request
                    .header("HTTP-Referer", "https://github.com/SuperDBA/SuperDBA")
                    .header("X-Title", "SuperDBA");
            }
            let response = request
                .send()
                .await
                .map_err(|error| format!("{provider} request failed: {error}"))?;
            let value = decode_response(response, provider).await?;
            parse_openai(&value)
        }
    }
}

/// MiniMax exposes an OpenAI-compatible endpoint in two regions. Try the region
/// that worked last time first, and remember whichever one answers.
async fn minimax_complete(
    client: &reqwest::Client,
    api_key: &str,
    body: &Value,
) -> Result<AssistantReply, String> {
    let mut errors = Vec::new();
    for endpoint in minimax_endpoint_candidates() {
        match post_minimax(client, api_key, endpoint, body).await {
            Ok(reply) => {
                remember_minimax_endpoint(endpoint);
                return Ok(reply);
            }
            Err(error) => errors.push(error),
        }
    }
    Err(format!("MiniMax request failed: {}", errors.join("; ")))
}

async fn post_minimax(
    client: &reqwest::Client,
    api_key: &str,
    endpoint: MiniMaxEndpoint,
    body: &Value,
) -> Result<AssistantReply, String> {
    let response = client
        .post(format!("{}/chat/completions", endpoint.openai_base_url))
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(body)
        .send()
        .await
        .map_err(|error| format!("{}: {error}", endpoint.region))?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!(
            "{} ({status}): {}",
            endpoint.region,
            truncate(&text, 2000)
        ));
    }
    let value = response
        .json::<Value>()
        .await
        .map_err(|error| format!("Invalid MiniMax response: {error}"))?;
    parse_openai(&value)
}

fn openai_endpoint(provider: &str, config: &AppConfig) -> Result<String, String> {
    Ok(match provider {
        "openai" => "https://api.openai.com/v1/chat/completions".to_string(),
        "deepseek" => "https://api.deepseek.com/v1/chat/completions".to_string(),
        "openrouter" => "https://openrouter.ai/api/v1/chat/completions".to_string(),
        "requesty" => "https://router.requesty.ai/v1/chat/completions".to_string(),
        "custom-openai" => {
            let base_url = config
                .ai_custom_openai_url
                .as_deref()
                .filter(|url| !url.is_empty())
                .ok_or("Custom OpenAI URL not configured.")?;
            crate::ai::build_api_url(base_url, "/chat/completions")
        }
        other => return Err(format!("Unsupported provider: {other}")),
    })
}

async fn decode_response(response: reqwest::Response, label: &str) -> Result<Value, String> {
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!(
            "{label} error ({status}): {}",
            truncate(&text, 2000)
        ));
    }
    response
        .json::<Value>()
        .await
        .map_err(|error| format!("Invalid {label} response: {error}"))
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut bounded: String = text.chars().take(limit).collect();
    bounded.push('…');
    bounded
}

/// OpenAI tool arguments arrive as a JSON string; Ollama may send an object.
pub(super) fn parse_arguments(value: &Value) -> Value {
    match value {
        Value::String(text) => {
            serde_json::from_str(text).unwrap_or_else(|_| json!({ "raw": text }))
        }
        Value::Null => json!({}),
        other => other.clone(),
    }
}

fn openai_messages(messages: &[ChatMessage], include_reasoning: bool) -> Vec<Value> {
    let mut wire = Vec::new();
    for message in messages {
        match message.role {
            ChatRole::System => {}
            ChatRole::User => wire.push(json!({"role": "user", "content": message.content})),
            ChatRole::Tool => wire.push(json!({
                "role": "tool",
                "tool_call_id": message.tool_call_id.clone().unwrap_or_default(),
                "content": message.content,
            })),
            ChatRole::Assistant => {
                let calls = message
                    .tool_calls
                    .iter()
                    .map(|call| {
                        json!({
                            "id": call.id,
                            "type": "function",
                            "function": {
                                "name": call.name,
                                "arguments": serde_json::to_string(&call.arguments).unwrap_or_else(|_| "{}".to_string()),
                            }
                        })
                    })
                    .collect::<Vec<_>>();
                let content = if message.content.is_empty() {
                    Value::Null
                } else {
                    Value::String(message.content.clone())
                };
                let mut entry = json!({"role": "assistant", "content": content});
                if include_reasoning {
                    if let Some(reasoning) = message
                        .reasoning
                        .as_deref()
                        .filter(|value| !value.is_empty())
                    {
                        entry["reasoning_content"] = Value::String(reasoning.to_string());
                    }
                }
                if !calls.is_empty() {
                    entry["tool_calls"] = Value::Array(calls);
                }
                wire.push(entry);
                for call in &message.tool_calls {
                    if let Some(result) = &call.result {
                        wire.push(json!({
                            "role": "tool",
                            "tool_call_id": call.id,
                            "content": result.to_string(),
                        }));
                    }
                }
            }
        }
    }
    wire
}

fn openai_tools(tools: &[ToolSchema]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "type": "function",
                "function": {
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input_schema,
                }
            })
        })
        .collect()
}

pub(super) fn openai_body(
    model: &str,
    system: &str,
    messages: &[ChatMessage],
    tools: &[ToolSchema],
) -> Value {
    openai_body_with(model, system, messages, tools, false)
}

pub(super) fn minimax_body(
    model: &str,
    system: &str,
    messages: &[ChatMessage],
    tools: &[ToolSchema],
) -> Value {
    openai_body_with(model, system, messages, tools, true)
}

fn openai_body_with(
    model: &str,
    system: &str,
    messages: &[ChatMessage],
    tools: &[ToolSchema],
    include_reasoning: bool,
) -> Value {
    let mut wire = vec![json!({"role": "system", "content": system})];
    wire.extend(openai_messages(messages, include_reasoning));
    let mut body = json!({
        "model": model,
        "messages": wire,
        "temperature": 0.0,
    });
    if !tools.is_empty() {
        body["tools"] = Value::Array(openai_tools(tools));
    }
    body
}

fn ollama_body(model: &str, system: &str, messages: &[ChatMessage], tools: &[ToolSchema]) -> Value {
    let mut body = openai_body(model, system, messages, tools);
    body["stream"] = json!(false);
    body
}

pub(super) fn parse_openai(value: &Value) -> Result<AssistantReply, String> {
    let message = value
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .ok_or("Invalid response format: missing choices[0].message")?;
    let content = message
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let reasoning = message
        .get("reasoning_content")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let mut calls = Vec::new();
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
            let id = entry
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("call_{}", uuid::Uuid::new_v4()));
            let arguments = entry
                .pointer("/function/arguments")
                .cloned()
                .unwrap_or(Value::Null);
            calls.push(ProviderToolCall {
                id,
                name,
                arguments: parse_arguments(&arguments),
            });
        }
    }
    Ok(AssistantReply {
        content,
        reasoning,
        tool_calls: calls,
        notice: None,
    })
}

fn parse_ollama(value: &Value) -> Result<AssistantReply, String> {
    let message = value
        .get("message")
        .ok_or("Invalid response format: missing message")?;
    let content = message
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut calls = Vec::new();
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
            calls.push(ProviderToolCall {
                id: format!("call_{}", uuid::Uuid::new_v4()),
                name,
                arguments: parse_arguments(&arguments),
            });
        }
    }
    Ok(AssistantReply {
        content,
        reasoning: None,
        tool_calls: calls,
        notice: None,
    })
}

fn anthropic_messages(messages: &[ChatMessage]) -> Vec<Value> {
    let mut wire: Vec<Value> = Vec::new();
    for message in messages {
        match message.role {
            ChatRole::System => {}
            ChatRole::User => {
                wire.push(json!({"role": "user", "content": message.content}));
            }
            ChatRole::Tool => wire.push(json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": message.tool_call_id.clone().unwrap_or_default(),
                    "content": message.content,
                }]
            })),
            ChatRole::Assistant => {
                let mut blocks = Vec::new();
                if !message.content.is_empty() {
                    blocks.push(json!({"type": "text", "text": message.content}));
                }
                for call in &message.tool_calls {
                    blocks.push(json!({
                        "type": "tool_use",
                        "id": call.id,
                        "name": call.name,
                        "input": call.arguments,
                    }));
                }
                if blocks.is_empty() {
                    continue;
                }
                wire.push(json!({"role": "assistant", "content": blocks}));
                let results = message
                    .tool_calls
                    .iter()
                    .filter_map(|call| {
                        call.result.as_ref().map(|result| {
                            json!({
                                "type": "tool_result",
                                "tool_use_id": call.id,
                                "content": result.to_string(),
                            })
                        })
                    })
                    .collect::<Vec<_>>();
                if !results.is_empty() {
                    wire.push(json!({"role": "user", "content": results}));
                }
            }
        }
    }
    wire
}

fn anthropic_tools(tools: &[ToolSchema]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.input_schema,
            })
        })
        .collect()
}

pub(super) fn anthropic_body(
    model: &str,
    system: &str,
    messages: &[ChatMessage],
    tools: &[ToolSchema],
) -> Value {
    let mut body = json!({
        "model": model,
        "system": system,
        "messages": anthropic_messages(messages),
        "max_tokens": 2048,
    });
    if !tools.is_empty() {
        body["tools"] = Value::Array(anthropic_tools(tools));
    }
    body
}

pub(super) fn parse_anthropic(value: &Value) -> Result<AssistantReply, String> {
    let blocks = value
        .get("content")
        .and_then(Value::as_array)
        .ok_or("Invalid response format: missing content blocks")?;
    let mut text = String::new();
    let mut calls = Vec::new();
    for block in blocks {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(value) = block.get("text").and_then(Value::as_str) {
                    text.push_str(value);
                }
            }
            Some("tool_use") => {
                let name = block
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                if name.is_empty() {
                    continue;
                }
                let id = block
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("call_{}", uuid::Uuid::new_v4()));
                calls.push(ProviderToolCall {
                    id,
                    name,
                    arguments: block.get("input").cloned().unwrap_or_else(|| json!({})),
                });
            }
            _ => {}
        }
    }
    Ok(AssistantReply {
        content: text,
        reasoning: None,
        tool_calls: calls,
        notice: None,
    })
}

/// Result of a streaming turn.
#[derive(Debug)]
pub enum StreamOutcome {
    Reply(AssistantReply),
    /// The user stopped the turn after `text` had already streamed.
    Cancelled(AssistantReply),
}

/// How long a stream read waits before re-checking the cancellation flag.
const STREAM_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(500);

/// Streams a completion, forwarding text deltas through `on_text` and polling
/// `is_cancelled` between chunks. Providers without a streaming adapter fall
/// back to the buffered call and simply emit no deltas.
#[allow(clippy::too_many_arguments)]
pub async fn complete_streaming(
    runtime: &RuntimeContext,
    cache: &CredentialCache,
    config: &AppConfig,
    provider: &str,
    model: &str,
    system: &str,
    messages: &[ChatMessage],
    tools: &[ToolSchema],
    options: GenerationOptions,
    on_text: &mut (dyn FnMut(&str) + Send),
    is_cancelled: &(dyn Fn() -> bool + Send + Sync),
) -> Result<StreamOutcome, String> {
    if !supports_tool_calling(provider) {
        return Err(UNSUPPORTED_TOOL_CALLING.to_string());
    }
    match provider {
        "minimax" => {
            let api_key = get_api_key(runtime, cache, provider)?;
            let client = crate::proxy::ai_http_client(provider)?;
            let mut body = minimax_body(model, system, messages, tools);
            apply_generation_options(&mut body, provider, options);
            Ok(StreamOutcome::Reply(
                minimax_complete(&client, &api_key, &body).await?,
            ))
        }
        "anthropic" => {
            let api_key = get_api_key(runtime, cache, provider)?;
            let client = crate::proxy::ai_http_client(provider)?;
            let mut body = anthropic_body(model, system, messages, tools);
            apply_generation_options(&mut body, provider, options);
            body["stream"] = json!(true);
            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|error| format!("Anthropic request failed: {error}"))?;
            let response = ensure_stream_response(response, "Anthropic").await?;
            let mut state = super::stream::AnthropicStream::default();
            let cancelled = consume_stream(response, is_cancelled, |line| {
                let Some(data) = line.strip_prefix("data:") else {
                    return Ok(true);
                };
                let data = data.trim();
                if data.is_empty() {
                    return Ok(true);
                }
                if let Ok(event) = serde_json::from_str::<Value>(data) {
                    if let Some(delta) = state.apply(&event) {
                        on_text(&delta);
                    }
                    if event.get("type").and_then(Value::as_str) == Some("message_stop") {
                        return Ok(false);
                    }
                }
                Ok(true)
            })
            .await?;
            if cancelled {
                return Ok(StreamOutcome::Cancelled(state.finish()));
            }
            Ok(StreamOutcome::Reply(state.finish()))
        }
        "ollama" => {
            let client = crate::proxy::direct_http_client()?;
            let port = config.ai_ollama_port.unwrap_or(11434);
            let mut body = ollama_body(model, system, messages, tools);
            apply_generation_options(&mut body, provider, options);
            body["stream"] = json!(true);
            let response = client
                .post(format!("http://localhost:{port}/api/chat"))
                .json(&body)
                .send()
                .await
                .map_err(|error| format!("Failed to connect to Ollama on port {port}: {error}"))?;
            let response = ensure_stream_response(response, "Ollama").await?;
            let mut state = super::stream::OllamaStream::default();
            let cancelled = consume_stream(response, is_cancelled, |line| {
                if let Ok(value) = serde_json::from_str::<Value>(line) {
                    if let Some(message) = value.get("message") {
                        if let Some(delta) = state.apply(message) {
                            on_text(&delta);
                        }
                    }
                    if value.get("done").and_then(Value::as_bool) == Some(true) {
                        return Ok(false);
                    }
                }
                Ok(true)
            })
            .await?;
            if cancelled {
                return Ok(StreamOutcome::Cancelled(state.finish()));
            }
            Ok(StreamOutcome::Reply(state.finish()))
        }
        _ => {
            let api_key = get_api_key(runtime, cache, provider)?;
            let client = crate::proxy::ai_http_client(provider)?;
            let url = openai_endpoint(provider, config)?;
            let mut body = openai_body(model, system, messages, tools);
            apply_generation_options(&mut body, provider, options);
            body["stream"] = json!(true);
            let mut request = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body);
            if !api_key.is_empty() {
                request = request.header("Authorization", format!("Bearer {api_key}"));
            }
            if matches!(provider, "openrouter" | "requesty") {
                request = request
                    .header("HTTP-Referer", "https://github.com/SuperDBA/SuperDBA")
                    .header("X-Title", "SuperDBA");
            }
            let response = request
                .send()
                .await
                .map_err(|error| format!("{provider} request failed: {error}"))?;
            let response = ensure_stream_response(response, provider).await?;
            let mut state = super::stream::OpenAiStream::default();
            let cancelled = consume_stream(response, is_cancelled, |line| {
                let Some(data) = line.strip_prefix("data:") else {
                    return Ok(true);
                };
                let data = data.trim();
                if data == "[DONE]" {
                    return Ok(false);
                }
                if data.is_empty() {
                    return Ok(true);
                }
                if let Ok(chunk) = serde_json::from_str::<Value>(data) {
                    if let Some(delta) = state.apply(&chunk) {
                        on_text(&delta);
                    }
                }
                Ok(true)
            })
            .await?;
            if cancelled {
                return Ok(StreamOutcome::Cancelled(state.finish()));
            }
            Ok(StreamOutcome::Reply(state.finish()))
        }
    }
}

async fn ensure_stream_response(
    response: reqwest::Response,
    label: &str,
) -> Result<reqwest::Response, String> {
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!(
            "{label} error ({status}): {}",
            truncate(&text, 2000)
        ));
    }
    Ok(response)
}

/// Reads lines from a streaming HTTP response. Returns `Ok(true)` when the
/// caller cancelled the turn.
async fn consume_stream<F>(
    response: reqwest::Response,
    is_cancelled: &(dyn Fn() -> bool + Send + Sync),
    mut on_line: F,
) -> Result<bool, String>
where
    F: FnMut(&str) -> Result<bool, String> + Send,
{
    use futures::StreamExt;

    let mut stream = Box::pin(response.bytes_stream());
    let mut buffer = String::new();
    loop {
        if is_cancelled() {
            return Ok(true);
        }
        match tokio::time::timeout(STREAM_POLL_INTERVAL, stream.next()).await {
            Err(_) => continue,
            Ok(None) => {
                for line in super::stream::take_lines(&mut buffer) {
                    if !on_line(&line)? {
                        return Ok(false);
                    }
                }
                let remainder = buffer.trim().to_string();
                if !remainder.is_empty() {
                    on_line(&remainder)?;
                }
                return Ok(false);
            }
            Ok(Some(Err(error))) => return Err(format!("The response stream failed: {error}")),
            Ok(Some(Ok(bytes))) => {
                buffer.push_str(&String::from_utf8_lossy(&bytes));
                for line in super::stream::take_lines(&mut buffer) {
                    if !on_line(&line)? {
                        return Ok(false);
                    }
                }
            }
        }
    }
}
