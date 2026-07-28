+++
name = "llm_provider_call"
agent = "haplotes"

[description]
en = "Call an external LLM provider API"
zh-Hans = "调用外部 LLM 提供商 API"
zh-Hant = "呼叫外部 LLM 提供者 API"
ja = "外部LLMプロバイダーAPIを呼び出し"
ko = "외부 LLM 제공자 API 호출"
fr = "Appeler une API de fournisseur LLM externe"
es = "Llamar a una API de proveedor LLM externo"
ru = "Вызвать API внешнего провайдера LLM"
+++

# llm_provider_call

## Description

Sends a chat completion request to a specified LLM provider API and returns the generated response. This is the primary tool for haplotes to route inference calls to providers such as OpenAI, Anthropic, Google, and local/self-hosted endpoints, handling authentication and response normalization transparently.

## Parameters

- **tier** (string, optional): The model tier to use. Accepted values: `basic`, `standard`, `premium`, `reasoning`. Default: `basic`
- **messages** (array, required, separate-call): Conversation messages. Provide via `llm_provider_call.messages("...")` in a follow-up call. Pass as a JSON array of objects with `role` and `content` keys.
- **temperature** (number, optional): Sampling temperature between 0.0 and 2.0. Controls randomness of the output. Default: 1.0
- **`max_tokens`** (number, optional): Maximum number of tokens to generate in the response

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Returns

### Success

```text
Operation successful

Provider: anthropic
Model: claude-3-opus-20240229

Response:
The analysis shows that the code follows a modular architecture...

Usage:
  Input tokens: 1523
  Output tokens: 342
  Total tokens: 1865

Finish reason: stop
```

### Failure

```text
Operation failed

Error: Model not found
Provider: openai
Model: gpt-5-turbo
Message: The specified model does not exist or is not available for your account.
```

## Examples

### Example 1: Simple chat completion (multi-step)

```text
# Step 1: structured parameters
llm_provider_call({ "provider": "openai", "model": "gpt-4o", "temperature": 0.7, "max_tokens": 200 })
# Step 2: messages via separate call
llm_provider_call.messages('[{"role": "user", "content": "Explain quantum computing in one paragraph"}]')
```

### Example 2: System prompt with conversation context (multi-step)

```text
# Step 1
llm_provider_call({ "provider": "anthropic", "model": "claude-3-opus", "temperature": 0.2 })
# Step 2
llm_provider_call.messages('[{"role": "system", "content": "You are a security auditor."}, {"role": "user", "content": "Review this code for SQL injection."}]')
```

### Example 3: Low-temperature generation for deterministic output

```text
provider: "google"
model: "gemini-pro"
messages: r#"[{"role": "user", "content": "Classify this text as positive or negative: 'The product exceeded my expectations'"}]"#
temperature: 0.0
max_tokens: 10
```

## Important Notes

- **Provider is required**: The `provider` parameter must be explicitly specified — there is no auto-detection fallback
- **Token limits**: Each model has a different context window. Requests exceeding the limit will fail with a clear error message
- **Rate limiting**: Frequent calls may hit provider rate limits. Implement backoff logic in the calling agent
- **Cost awareness**: Different models and providers have different pricing. Monitor token usage returned in the response metadata
- **Error handling**: Always check the `finish_reason` in the response. A value of `length` indicates the output was truncated due to the `max_tokens` limit
