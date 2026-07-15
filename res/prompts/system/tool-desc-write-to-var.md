+++
id = "tool-desc-write-to-var"
title = "write_to_var 工具描述"
kind = "tool_description"
tool = "write_to_var"
+++

Store a string in vars['`var_name`']. MANDATORY for ANY text > ~100 chars: file contents, report bodies, generated code, base64 blobs, template strings. Always prefer over inlining in exec. MUST reference as vars['`var_name`'] or vars.var_name in exec — NOT as a bare name. Persists across calls.
