/**
 * Shared chat/LLM types for the plana component kit.
 *
 * Deliberately dependency-light: plain data shapes only, so both arona
 * and shittim-chest can map their own stores onto these props without
 * importing plana_llm_provider internals.
 */

/** Message role — drives bubble alignment and tinting. */
export type PChatRole = "user" | "assistant";

/** Lifecycle of a tool call block. */
export type PMcpToolCallStatus = "pending" | "running" | "done" | "error";

/** Tool call payload rendered by PMcpToolBlock (also nested in PChatMessage). */
export interface PMcpToolCall {
  id?: string;
  toolName: string;
  agentType?: string;
  status: PMcpToolCallStatus;
  callText?: string;
  resultText?: string;
  durationMs?: number;
  defaultExpanded?: boolean;
}

/** Voice popup phase — see PVoiceInputPopup. */
export type PVoicePopupMode = "notConfigured" | "listening" | "transcribing";

/**
 * Voice state fed by the parent (e.g. from a useVoiceInput composable).
 * `open` + `mode` drive the anchored PVoiceInputPopup; `transcribing`
 * disables the mic button while recognition is running.
 */
export interface PVoiceState {
  open: boolean;
  mode: PVoicePopupMode;
  transcribing?: boolean;
}

/** Attachment row in the PRichInput strip. Parent-owned upload state. */
export interface PAttachmentItem {
  id: string;
  name: string;
  type: string;
  size: number;
  preview?: string;
  /** Upload progress 0-100; omitted/100 = ready. */
  progress?: number;
  status?: "uploading" | "done" | "error";
}

/** Attachment payload for PAttachmentModal preview. */
export interface PAttachmentDetail {
  name: string;
  type: string;
  size: number;
  preview?: string;
  url?: string;
  /** Original File handle — used to read text/code content for preview. */
  file?: File;
}

/** One model row in the PTokenUsagePanel per-model breakdown. */
export interface PModelUsageEntry {
  model: string;
  tokenCount: number;
}

/** Read-only cost inputs for PTokenUsagePanel (USD amounts). */
export interface PModelCosts {
  prompt: number;
  completion: number;
  cached?: number;
}

/** Status of a model download rendered by PModelDownloadCard. */
export type PModelDownloadStatus = "pending" | "downloading" | "done" | "error";
