// Every export line references a concrete generated file; the bindings
// folders carry no per-directory index.ts, so directory-star exports would
// dangle. A handful of distinct generated types share names across files;
// the flat-surface winner is resolved by named-export EXCLUSION LISTS, NOT
// by TypeScript star-export precedence:
//
//   - The tools namespace files below that collide with earlier star-exported
//     files (aporia, kalos, neikos, orexis) are re-exported via an explicit
//     `export { ... } from` list instead of `export *`. The list is a FULL
//     enumeration of what that file contributes to the flat surface — a
//     name in the generated file but not in the list is excluded.
//   - Each list therefore shadows its file's colliding duplicates on
//     purpose: the name the flat surface resolves to comes from whichever
//     OTHER (star-exported) source the earlier line exports, so "first
//     listed source wins" is the deliberate outcome, not an artifact.
//   - Shadowed duplicates stay importable via deep imports
//     (`import type { FileEntry } from "@celestia-island/plana-celestia-types/bindings/tools/kalos"`).
//
// The 6 current collisions (flat winner -> shadowed source):
//
//   WorkspaceStatusParams    ws/workspace        wins vs tools/aporia
//   FileEntry                httpTypes           wins vs tools/kalos
//   FileReadParams           ws/fileBrowsing     wins vs tools/kalos
//   FileTreeEntry            ws/fileBrowsing     wins vs tools/kalos
//   ContainerSnapshotParams  ws/stateSync        wins vs tools/neikos
//   ReportHumanParams        tools/hubris          wins vs tools/orexis
//
// MAINTENANCE WARNING: the four exclusion lists are hand-maintained here —
// there is no generator script that rewrites this file. New types added to
// the generated aporia/kalos/neikos/orexis files do NOT appear in the flat
// surface unless their name is appended to the matching `export { ... }`
// line below (keep the list alphabetically sorted). A type added with a
// name that already exists on the flat surface is silently shadowed: it
// must either stay deep-import-only (deliberate shadow) or the existing
// name must be dropped from the earlier star-exported source.
export * from "./bindings/engine";
export * from "./bindings/enums";
export * from "./bindings/httpTypes";
export * from "./bindings/mdd";
export * from "./bindings/model";
export * from "./bindings/ws/agentLifecycle";
export * from "./bindings/ws/auth";
export * from "./bindings/ws/bridgeNetwork";
export * from "./bindings/ws/core";
export * from "./bindings/ws/fileBrowsing";
export * from "./bindings/ws/handshake";
export * from "./bindings/ws/industrial";
export * from "./bindings/ws/knowledgeBase";
export * from "./bindings/ws/layer2";
export * from "./bindings/ws/llmProvider";
export * from "./bindings/ws/logs";
export * from "./bindings/ws/malkuth";
export * from "./bindings/ws/noa";
export * from "./bindings/ws/stateSync";
export * from "./bindings/ws/systemUi";
export * from "./bindings/ws/tasks";
export * from "./bindings/ws/views";
export * from "./bindings/ws/workspace";
export * from "./bindings/ws/yolo";
export type { AnomalyDetectParams, AnomalyInfo, AnomalyResult, CausalReasonParams, CausalReasonResult, CorrelationInfo, Hypothesis, LlmChatParams, LlmChatResult, MediaAssetItem, MediaAssetRegisterResult, MediaAssetRetrieveResult, RagDbDeleteParams, RagDbDeleteResult, RagDbReadParams, RagDbReadResult, RagDbStatsParams, RagDbStatsResult, RagDbWriteParams, RagDbWriteResult, RagDocResult, TranslateReportParams, TranslateReportResult, WorkspaceIndexParams, WorkspaceIndexResult, WorkspaceSearchDoc, WorkspaceSearchParams, WorkspaceSearchResult, WorkspaceStatusResult } from "./bindings/tools/aporia";
export * from "./bindings/tools/eleos";
export * from "./bindings/tools/epieikeia";
export * from "./bindings/tools/haplotes";
export * from "./bindings/tools/hubris";
export type { Annotation, FileCreateDirParams, FileDeleteParams, FileDeleteResult, FileEditParams, FileEditResult, FileExistsParams, FileExistsResult, FileGetInfoParams, FileInfoResult, FileListParams, FileListResult, FileReadResult, FileTreeListResult, FileWriteParams, FileWriteResult, ListAnnotationsResult, MkDirResult, ResolveAnnotationResult } from "./bindings/tools/kalos";
export type { CheckWaitParams, ContainerCreateResult, ContainerFilterCriteria, ContainerForkParams, ContainerForkResult, ContainerInfoParams, ContainerInfoResult, ContainerListItem, ContainerListParams, ContainerListResult, ContainerRemoveParams, ContainerRemoveResult, ContainerSnapshotResult, ContainerStartParams, ContainerStartResult, ContainerStopParams, ContainerStopResult, ExecOnContainerParams, ExecResult, GitPushBranchParams, GitPushResult, NewContainerToolParams, NewContainerVolumeMount, SidecarDeliverResult, SidecarKillParams, SidecarSendParams, SidecarSendResult, SidecarSpawnParams, ToolchainEnsureParams, ToolchainEnsureResult, ToolchainListParams, ToolchainListResult, ToolchainProfileInfo, ToolchainVolumeSpec, VolumeInfo, WaitParams } from "./bindings/tools/neikos";
export type { AgentIntegrityParams, AskResult, AuditAlignmentParams, AuditAlignmentResult, AuditFinding, AuditLegalityParams, AuditLegalityResult, BlockToolParams, CheckResultItem, ComplianceReportParams, ComplianceReportToolResult, ComplianceRule, ComplianceSummary, InspectToolCallParams, ManageSensitivityRulesParams, ReplyResult, ReportDetail, ReportHumanResult, RuleCheckResult, SecurityAuditParams, SecurityStatusParams, SecuritySuggestionsParams, SensitivityRule, SetNetworkPolicyParams, SetRiskThresholdParams, SetSecurityPolicyParams, StandardCheckParams, StandardCheckResult, StandardRegisterResult, UnblockToolParams } from "./bindings/tools/orexis";
export type { GrantItem, GrantListResponse, MyPermissions, OAuthProvider, RbacUser, RbacUsersResponse } from "./bindings/protocol-core-httpTypes";
export * from "./bindings/tools/philia";
export * from "./bindings/tools/polemos";
export * from "./bindings/tools/skemma";
export * from "./bindings/tools/skopeo";
export * from "./bindings/tools/web_automation";
export * from "./bindings/serde_json/JsonValue";
