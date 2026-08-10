+++
name = "reconcile_rows"
agent = "data_grid"
execution_mode = "read"

[description]
en = "Synchronise table rows with a data source (sensors, alarms, project board)"
+++

# Reconcile Rows

Fetch the underlying source (station-sensor / station-alarm /
project-board via `view.fetch`), diff against the current table rows,
then apply `table.add_row` / `table.update_cell` / `table.delete_row`
edits so the table mirrors the source truth. Use `full_replace=false`
pushes for streaming updates.
