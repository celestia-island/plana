+++
id = "return-type-convention"
title = "返回类型约定"
kind = "reference"
+++

# Return Type Convention

This document is injected into every skill prompt at assembly time. It defines the standard return type behavior for all skills under the exec-only microkernel architecture.

## Return Type

The return type of this skill is defined by its corresponding `.d.ts` declaration, auto-generated from the skill's return struct at compile time and appended to this prompt at runtime.

## IEPL Type Enforcement

When this skill produces output (via `report()`, `report_human()`, or any other return mechanism), the IEPL runtime validates the return value against the declared type.

If validation fails, the skill receives an error containing the correct type constraint and must retry with corrected output. The skill does NOT terminate on type mismatch — it gets a chance to correct.
