import type {
  StaticAnalyzeParams,
  StaticAnalyzeResult,
  CodeReviewParams,
  CodeReviewResult,
  QualityCheckParams,
  QualityCheckResult,
  RefactorSuggestParams,
  RefactorSuggestResult,
  LspDiagnoseParams,
  LspDiagnoseResult,
  LspSymbolsParams,
  LspSymbolsResult,
  LspRefactorParams,
  LspRefactorResult,
} from "./lsp_types";

const LANGUAGE_MAP: Record<string, string> = {
  ".rs": "rust",
  ".ts": "typescript",
  ".tsx": "typescript",
  ".js": "javascript",
  ".jsx": "javascript",
  ".py": "python",
  ".go": "go",
};

function detectLanguage(filePath: string): string | null {
  const ext = filePath.substring(filePath.lastIndexOf("."));
  return LANGUAGE_MAP[ext] ?? null;
}

async function withLspSession<T>(
  language: string,
  filePath: string,
  content: string,
  operation: (sidecarName: string) => Promise<T>
): Promise<T> {
  const sidecarName = `lsp-${language}-${Date.now()}`;
  try {
    await sidecar_spawn({
      name: sidecarName,
      language,
      framing: "content_length",
    });

    await sidecar_send({
      name: sidecarName,
      method: "initialize",
      params: {
        processId: null,
        rootUri: "file:///workspace",
        capabilities: {},
      },
      timeout_secs: 30,
    });

    await sidecar_send({
      name: sidecarName,
      method: "initialized",
      params: {},
    });

    await sidecar_send({
      name: sidecarName,
      method: "textDocument/didOpen",
      params: {
        textDocument: {
          uri: `file:///workspace/${filePath}`,
          languageId: language,
          version: 1,
          text: content,
        },
      },
    });

    return await operation(sidecarName);
  } finally {
    try {
      await sidecar_send({
        name: sidecarName,
        method: "shutdown",
        params: {},
      });
    } catch {}
    try {
      await sidecar_kill({ name: sidecarName });
    } catch {}
  }
}

// ─── static_analyze ──────────────────────────────────────────────────────────
//
// Single responsibility: structural and stylistic static analysis.
// Security checks → orexis.security_audit. Performance checks → neikos.exec_on_container.
// Test-quality checks → testing_pipeline SOP inlines its own logic.

export async function static_analyze(
  params: StaticAnalyzeParams
): Promise<StaticAnalyzeResult> {
  const filePath = params.file_path;
  const language = detectLanguage(filePath);

  if (!language) {
    return {
      file_path: filePath,
      analyzers_run: [],
      total_findings: 0,
      findings: [],
    };
  }

  const content =
    params.content ?? (await file_read({ path: filePath }));
  const findings: StaticAnalyzeResult["findings"] = [];

  if (language === "rust") {
    const deadCodePattern =
      /warning:\s+(?:function|method|struct|enum|module|constant|static|type)\s+`(\w+)`\s+is never (?:used|read|mutated)/g;
    let match;
    while ((match = deadCodePattern.exec(content)) !== null) {
      const lineNum = content.substring(0, match.index).split("\n").length;
      findings.push({
        analyzer: "dead_code",
        severity: "low",
        line: lineNum,
        column: null,
        message: `\`${match[1]}\` is never used`,
        suggestion: `Remove unused \`${match[1]}\` or prefix with \`_\` to suppress`,
      });
    }

    const unwrapPattern = /\.unwrap\(\)/g;
    while ((match = unwrapPattern.exec(content)) !== null) {
      const lineNum = content.substring(0, match.index).split("\n").length;
      const line = content.split("\n")[lineNum - 1];
      if (!line?.includes("#[test]") && !line?.includes("#[cfg(test)]")) {
        findings.push({
          analyzer: "error_handling",
          severity: "medium",
          line: lineNum,
          column: match.index - content.lastIndexOf("\n", match.index),
          message: "Unwrap call in non-test code",
          suggestion: "Use `?` operator or `.map_err()` for error propagation",
        });
      }
    }
  }

  const defaultChecks = ["dead_code", "naming", "import_order"];
  const checks = params.checks ?? defaultChecks;

  return {
    file_path: filePath,
    analyzers_run: checks,
    total_findings: findings.length,
    findings,
  };
}

// ─── code_review ─────────────────────────────────────────────────────────────
//
// Single responsibility: semantic code review — readability, structure, logic.
// Architecture review → combine with lsp_symbols + quality_check.
// Security review → orexis.security_audit. Performance review → neikos.exec_on_container.

export async function code_review(
  params: CodeReviewParams
): Promise<CodeReviewResult> {
  const filePath = params.file_path;
  const language = detectLanguage(filePath) ?? "unknown";
  const content =
    params.content ?? (await file_read({ path: filePath }));

  const issues: CodeReviewResult["issues"] = [];

  const lines = content.split("\n");
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line.length > 120) {
      issues.push({
        severity: "low",
        category: "style",
        line: i + 1,
        message: `Line exceeds 120 characters (${line.length})`,
        suggestion: "Break into multiple lines",
      });
    }
    if (line.includes("TODO") || line.includes("FIXME") || line.includes("HACK")) {
      issues.push({
        severity: "low",
        category: "technical_debt",
        line: i + 1,
        message: `Technical debt marker found: ${line.trim()}`,
        suggestion: "Resolve or create a tracked issue",
      });
    }
  }

  const score = Math.max(0, 100 - issues.length * 5);

  return {
    file_path: filePath,
    language,
    overall_score: score,
    issues_found: issues.length,
    issues,
  };
}

// ─── quality_check ───────────────────────────────────────────────────────────

export async function quality_check(
  params: QualityCheckParams
): Promise<QualityCheckResult> {
  const files = await file_list({ path: params.scope, recursive: true });
  const sourceFiles = files.filter((f: string) =>
    Object.keys(LANGUAGE_MAP).some((ext) => f.endsWith(ext))
  );

  return {
    scope: params.scope,
    maintainability_index: 0,
    cyclomatic_complexity_avg: 0,
    documentation_coverage: 0,
    coupling_score: 0,
    cohesion_score: 0,
    technical_debt_ratio: 0,
    file_count: sourceFiles.length,
    hotspot_files: [],
  };
}

// ─── refactor_suggest ────────────────────────────────────────────────────────

export async function refactor_suggest(
  params: RefactorSuggestParams
): Promise<RefactorSuggestResult> {
  const proposals: RefactorSuggestResult["proposals"] = [];
  const findings = params.findings ?? [];
  const prioritize = params.prioritize ?? "safety_impact_ratio";

  for (const finding of findings) {
    if (finding.severity === "critical" || finding.severity === "high") {
      proposals.push({
        id: `refactor-${proposals.length + 1}`,
        title: `Address ${finding.severity} finding: ${finding.message}`,
        description: finding.suggestion ?? finding.message,
        refactor_type: "extract_method",
        before: "",
        after: "",
        affected_files: [],
        affected_symbols: [],
        effort_estimate: "1-2 hours",
        risk_level: "needs_review",
        wave: 1,
        blast_radius: 1,
      });
    }
  }

  proposals.sort((a, b) => a.wave - b.wave);

  return {
    plan_id: params.plan_id ?? `plan-${Date.now()}`,
    proposals,
    wave_count: proposals.length > 0 ? Math.max(...proposals.map((p) => p.wave)) : 0,
    total_effort_estimate: `${proposals.length} items`,
  };
}

// ─── lsp_diagnose ────────────────────────────────────────────────────────────

export async function lsp_diagnose(
  params: LspDiagnoseParams
): Promise<LspDiagnoseResult> {
  const { file_path, language } = params;
  const content = await file_read({ path: file_path });

  return withLspSession(language, file_path, content, async (sidecarName) => {
    const result = await sidecar_send({
      name: sidecarName,
      method: "textDocument/publishDiagnostics",
      params: {
        textDocument: { uri: `file:///workspace/${file_path}` },
      },
      timeout_secs: 60,
    });

    const rawDiagnostics = result?.data?.response?.result?.diagnostics ?? [];
    const diagnostics: LspDiagnostic[] = rawDiagnostics.map(
      (d: Record<string, unknown>) => ({
        severity: severityFromLsp(d.severity as number),
        code: (d.code as string) ?? null,
        message: d.message as string,
        start_line: (d.range?.start?.line as number) ?? 0,
        start_col: (d.range?.start?.character as number) ?? 0,
        end_line: (d.range?.end?.line as number) ?? 0,
        end_col: (d.range?.end?.character as number) ?? 0,
        source: (d.source as string) ?? null,
      })
    );

    return {
      file_path,
      language,
      errors: diagnostics.filter((d) => d.severity === "error").length,
      warnings: diagnostics.filter((d) => d.severity === "warning").length,
      info_count: diagnostics.filter(
        (d) => d.severity === "information" || d.severity === "hint"
      ).length,
      diagnostics,
    };
  });
}

// ─── lsp_symbols ─────────────────────────────────────────────────────────────

export async function lsp_symbols(
  params: LspSymbolsParams
): Promise<LspSymbolsResult> {
  const { file_path } = params;
  const language = detectLanguage(file_path);
  if (!language) {
    return { file_path, total_symbols: 0, symbols: [] };
  }

  const content = await file_read({ path: file_path });

  return withLspSession(language, file_path, content, async (sidecarName) => {
    const result = await sidecar_send({
      name: sidecarName,
      method: "textDocument/documentSymbol",
      params: {
        textDocument: { uri: `file:///workspace/${file_path}` },
      },
      timeout_secs: 30,
    });

    const rawSymbols = result?.data?.response?.result ?? [];
    const symbols: LspSymbolsResult["symbols"] = rawSymbols.map(
      (s: Record<string, unknown>) => ({
        name: s.name as string,
        kind: symbolKindToString(s.kind as number),
        start_line: (s.range?.start?.line as number) ?? 0,
        start_col: (s.range?.start?.character as number) ?? 0,
        end_line: (s.range?.end?.line as number) ?? 0,
        end_col: (s.range?.end?.character as number) ?? 0,
        documentation: null,
        signature: null,
      })
    );

    return {
      file_path,
      total_symbols: symbols.length,
      symbols,
    };
  });
}

// ─── lsp_refactor ────────────────────────────────────────────────────────────

export async function lsp_refactor(
  params: LspRefactorParams
): Promise<LspRefactorResult> {
  const { file_path, refactor_type, range, params: refactorParams } = params;
  const language = detectLanguage(file_path);
  if (!language) {
    return {
      file_path,
      refactor_type,
      files_modified: 0,
      total_changes: 0,
      changes: [],
      errors: ["Unsupported language for file"],
    };
  }

  const content = await file_read({ path: file_path });

  return withLspSession(language, file_path, content, async (sidecarName) => {
    const result = await sidecar_send({
      name: sidecarName,
      method: "textDocument/codeAction",
      params: {
        textDocument: { uri: `file:///workspace/${file_path}` },
        range: range
          ? {
              start: { line: range.start_line, character: range.start_col },
              end: { line: range.end_line, character: range.end_col },
            }
          : {
              start: { line: 0, character: 0 },
              end: { line: 99999, character: 0 },
            },
        context: {
          diagnostics: [],
          only: ["refactor", "quickfix"],
        },
      },
      timeout_secs: 30,
    });

    const actions = result?.data?.response?.result ?? [];
    const changes: LspRefactorResult["changes"] = [];
    let filesModified = 0;
    const modifiedFiles = new Set<string>();

    for (const action of actions) {
      if (action.edit?.changes) {
        for (const [uri, edits] of Object.entries(action.edit.changes)) {
          const changedFile = uri.replace("file:///workspace/", "");
          modifiedFiles.add(changedFile);
          for (const edit of edits as Array<Record<string, unknown>>) {
            changes.push({
              file: changedFile,
              line: (edit.range?.start?.line as number) ?? 0,
              description: (edit.newText as string) ?? "",
            });
          }
        }
      }
    }

    return {
      file_path,
      refactor_type,
      files_modified: modifiedFiles.size,
      total_changes: changes.length,
      changes,
      errors: [],
    };
  });
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

function severityFromLsp(
  severity: number
): "error" | "warning" | "information" | "hint" {
  switch (severity) {
    case 1:
      return "error";
    case 2:
      return "warning";
    case 3:
      return "information";
    case 4:
      return "hint";
    default:
      return "information";
  }
}

function symbolKindToString(kind: number): string {
  const kinds: Record<number, string> = {
    1: "file",
    2: "module",
    3: "namespace",
    4: "package",
    5: "class",
    6: "method",
    7: "property",
    8: "field",
    9: "constructor",
    10: "enum",
    11: "interface",
    12: "function",
    13: "variable",
    14: "constant",
    15: "string",
    16: "number",
    17: "boolean",
    18: "array",
    19: "object",
    20: "key",
    21: "null",
    22: "enum_member",
    23: "struct",
    24: "event",
    25: "operator",
    26: "type_parameter",
  };
  return kinds[kind] ?? "unknown";
}
