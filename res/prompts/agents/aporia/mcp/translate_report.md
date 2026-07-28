+++
name = "translate_report"
agent = "aporia"

[description]
en = "Translate text content to a target language via LLM"
zhs = "通过大语言模型将文本内容翻译为目标语言"
zht = "透過大型語言模型將文字內容翻譯為目標語言"
ja = "LLMを介してテキストコンテンツを対象言語に翻訳する"
ko = "LLM을 통해 텍스트 콘텐츠를 대상 언어로 번역"
fr = "Traduire le contenu textuel vers une langue cible via un LLM"
es = "Traducir contenido de texto a un idioma de destino mediante LLM"
ru = "Перевести текстовое содержимое на целевой язык через LLM"
+++

# translate_report

## Description

Translates the provided text content into the specified target language using the LLM. The translation preserves formatting, technical terminology, and contextual nuances. Useful for localizing reports, documentation, and agent output for multilingual audiences.

## Parameters

- **content** (string, required): The text content to translate.
- **`target_language`** (string, required): The target language code or name (e.g., `"en"`, `"zh-CN"`, `"ja"`, `"ko"`, `"fr"`).

## Returns

### On Success

```text
Translation successful

Target language: <language>
Source length: <number> characters
Translated length: <number> characters

<translated text>
```

### On Failure

```text
Translation failed

Error: <error message>
```

## Examples

### Example 1: Translate a report to Japanese

Invocation:

```text
translate_report
  content: "The server load exceeded 90% threshold at 14:32 UTC."
  target_language: "ja"
```

Return:

```text
Translation successful

Target language: ja
Source length: 53 characters
Translated length: 38 characters

サーバー負荷が14:32 UTCに90%のしきい値を超えました。
```

### Example 2: Translate to Simplified Chinese

Invocation:

```text
translate_report
  content: "Anomaly detected in CPU utilization pattern."
  target_language: "zh-CN"
```

Return:

```text
Translation successful

Target language: zh-CN
Source length: 45 characters
Translated length: 15 characters

CPU利用率模式中检测到异常。
```

## Important Notes

- **Language detection**: The source language is auto-detected; do not specify it.
- **Formatting preservation**: Markdown formatting, code blocks, and tables are preserved in the output.
- **Context awareness**: The LLM considers domain-specific terminology. For best results, provide complete sentences rather than isolated words.
- **Rate limiting**: Large texts may be chunked internally. Very long content (>10,000 characters) may experience increased latency.
