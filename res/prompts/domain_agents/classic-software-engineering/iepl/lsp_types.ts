// ─── Shared Types ────────────────────────────────────────────────────────────

export interface IssueSummary {
  severity: "critical" | "high" | "medium" | "low";
  category: string;
  line: number | null;
  message: string;
  suggestion: string | null;
}

// ─── static_analyze ──────────────────────────────────────────────────────────

export interface StaticFinding {
  analyzer: string;
  severity: "critical" | "high" | "medium" | "low";
  line: number | null;
  column: number | null;
  message: string;
  suggestion: string | null;
}

// Only structural + stylistic checks. Security → OreXis. Performance → NeiKos profiling. Tests → testing_pipeline SOP.
export type StaticAnalyzeCheck =
  | "dead_code"
  | "naming"
  | "import_order"
  | "structural_patterns"
  | "error_handling";

export interface StaticAnalyzeParams {
  file_path: string;
  content?: string;
  scope?: string;
  checks?: StaticAnalyzeCheck[];
}

export interface StaticAnalyzeResult {
  file_path: string;
  analyzers_run: string[];
  total_findings: number;
  findings: StaticFinding[];
}

// ─── code_review ─────────────────────────────────────────────────────────────

export interface CodeReviewParams {
  file_path: string;
  content?: string;
}

export interface CodeReviewResult {
  file_path: string;
  language: string;
  overall_score: number;
  issues_found: number;
  issues: IssueSummary[];
}

// ─── quality_check ───────────────────────────────────────────────────────────

export interface QualityCheckParams {
  scope: string;
  metrics?: string[];
  thresholds?: Record<string, number>;
}

export interface QualityMetrics {
  scope: string;
  maintainability_index: number;
  cyclomatic_complexity_avg: number;
  documentation_coverage: number;
  coupling_score: number;
  cohesion_score: number;
  technical_debt_ratio: number;
  file_count: number;
  hotspot_files: string[];
}

export type QualityCheckResult = QualityMetrics;

// ─── refactor_suggest ────────────────────────────────────────────────────────

export interface RefactorSuggestParams {
  findings?: StaticFinding[] | IssueSummary[];
  prioritize?: "safety_impact_ratio" | "risk_reduction" | "impact_ease_ratio";
  constraints?: string[];
  plan_id?: string;
}

export interface RefactorProposal {
  id: string;
  title: string;
  description: string;
  refactor_type: "extract_method" | "extract_class" | "replace_conditional" | "introduce_parameter_object" | "rename" | "inline" | "move";
  before: string;
  after: string;
  affected_files: string[];
  affected_symbols: string[];
  effort_estimate: string;
  risk_level: "mechanical" | "needs_review" | "needs_design";
  wave: number;
  blast_radius: number;
}

export interface RefactorPlan {
  plan_id: string;
  proposals: RefactorProposal[];
  wave_count: number;
  total_effort_estimate: string;
}

export type RefactorSuggestResult = RefactorPlan;

// ─── lsp_diagnose ────────────────────────────────────────────────────────────

export interface LspDiagnostic {
  severity: "error" | "warning" | "information" | "hint";
  code: string | null;
  message: string;
  start_line: number;
  start_col: number;
  end_line: number;
  end_col: number;
  source: string | null;
}

export interface LspDiagnoseParams {
  file_path: string;
  language: string;
  scope?: string;
}

export interface LspDiagnoseResult {
  file_path: string;
  language: string;
  errors: number;
  warnings: number;
  info_count: number;
  diagnostics: LspDiagnostic[];
}

// ─── lsp_symbols ─────────────────────────────────────────────────────────────

export interface SymbolInfo {
  name: string;
  kind: string;
  start_line: number;
  start_col: number;
  end_line: number;
  end_col: number;
  documentation: string | null;
  signature: string | null;
  children?: SymbolInfo[];
}

export interface LspSymbolsParams {
  file_path: string;
  depth?: "full" | "exports" | "references";
}

export interface LspSymbolsResult {
  file_path: string;
  total_symbols: number;
  symbols: SymbolInfo[];
}

// ─── lsp_refactor ────────────────────────────────────────────────────────────

export interface RefactorChange {
  file: string;
  line: number;
  description: string;
}

export interface LspRefactorParams {
  file_path: string;
  refactor_type: string;
  range?: {
    start_line: number;
    start_col: number;
    end_line: number;
    end_col: number;
  };
  params?: Record<string, unknown>;
}

export interface LspRefactorResult {
  file_path: string;
  refactor_type: string;
  files_modified: number;
  total_changes: number;
  changes: RefactorChange[];
  errors: string[];
}

// ─── Aggregated Report Types ─────────────────────────────────────────────────

export interface ConsolidatedFinding {
  file_path: string;
  line: number | null;
  severity: "critical" | "high" | "medium" | "low";
  category: "static" | "semantic" | "security" | "quality" | "architecture";
  source: string;
  message: string;
  suggestion: string | null;
  deduplicated: boolean;
}

export interface ReviewReport {
  summary: string;
  findings: ConsolidatedFinding[];
  severity_counts: Record<string, number>;
  merge_blocked: boolean;
  hotspot_files: string[];
}

export interface HealthCheckReport {
  summary: string;
  mechanical_fixes: number;
  design_decisions: number;
  architecture_items: number;
  findings: ConsolidatedFinding[];
  quick_wins: string[];
}

export interface ArchitectureReviewReport {
  summary: string;
  findings: ConsolidatedFinding[];
  dependency_graph: Record<string, string[]>;
  risk_ranked_modules: Array<{ module: string; risk: string; reason: string }>;
}

export interface OptimizationReport {
  summary: string;
  baseline_metrics: Record<string, number>;
  opportunities: Array<{
    title: string;
    expected_improvement: string;
    risk: string;
    verification: string;
  }>;
}

export interface TestPipelineReport {
  summary: string;
  pass_count: number;
  fail_count: number;
  skip_count: number;
  line_coverage_pct: number;
  branch_coverage_pct: number;
  quality_issues: string[];
  recommendations: string[];
}

export interface CleanupReport {
  summary: string;
  deleted_files: string[];
  ambiguous_files: string[];
  sensitive_files: string[];
  build_verified: boolean;
}

export interface SecurityAuditReport {
  summary: string;
  findings: ConsolidatedFinding[];
  critical_findings: ConsolidatedFinding[];
  dependency_vulnerabilities: number;
  secrets_exposed: number;
}

export interface I18nCoverageReport {
  summary: string;
  base_language: string;
  coverage_matrix: Record<string, {
    total: number;
    translated: number;
    stale: number;
    drifted: number;
    pct: number;
  }>;
  items_requiring_human: string[];
}
