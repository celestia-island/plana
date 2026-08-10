+++
name = "tune_and_debug"
agent = "media_flow"
execution_mode = "read"

[description]
en = "Diagnose and tune a failing media pipeline"
+++

# Tune and Debug

When a pipeline node errors (see the node `error` state), inspect the
trace, adjust node params (quality, model, thresholds, max_iterations),
and re-run. For vision-critique loops, use the critique JSON to decide
refinement rounds; surface backend availability problems (TRELLIS not
deployed, API key missing) to the user instead of papering over them.
