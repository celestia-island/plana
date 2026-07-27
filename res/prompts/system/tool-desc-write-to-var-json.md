+++
id = "tool-desc-write-to-var-json"
title = "write_to_var_json 工具描述"
kind = "tool_description"
tool = "write_to_var_json"
+++

Store a validated JSON value as a parsed JS object in vars['`var_name`']. Content MUST be valid JSON — invalid JSON fails immediately with a clear parse error. The stored value is a real JS object/array (NOT a string). Reference as vars['`var_name`'] directly in exec.
