+++
id = "error-container-exec-too-long"
title = "容器执行超时错误"
kind = "error_template"
context = "container_exec_too_long"
+++

exec code is {{`code_len`}} chars (limit: {{limit}}). This limit is informational — the hard protections are at the container/Docker boundary via resource limits, circuit breakers, and rate limiting. If this message appears, split into multiple calls: 1) use `write_to_var` to store long data, 2) exec short code referencing `vars['var_name']`, 3) call `report()` (imported from `'hubris'`). JS context persists across calls.
