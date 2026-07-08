+++
id = "error-exec-syntax-invalid"
title = "执行语法无效错误"
kind = "error_template"
context = "exec_syntax_invalid"
+++

Code failed syntax validation: {{error}}. This often means the LLM output was truncated. Split your code into multiple smaller exec calls — the JS context is persistent, variables survive across calls. Use `write_to_var` for DATA only (text, JSON), NOT for executable code. First 80 chars: {{preview}}
