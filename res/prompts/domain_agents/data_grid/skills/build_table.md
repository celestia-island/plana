+++
name = "build_table"
agent = "data_grid"
execution_mode = "read"

[description]
en = "Create a multidimensional table from a natural-language request"
+++

# Build Table

Interpret the user's request into a typed table: decide the field
schema (`text` / `number` / `select` / `status` with options), then
materialise the view via the panel creation flow (`view.create` with
kind `data-table` + source purpose) and populate initial rows with
`table_push_data`.
