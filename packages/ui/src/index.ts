export { PAuthCard } from "./components/PlanaAuthCard";
export { PAdminShell } from "./components/PlanaAdminShell";
export { PAdminHeader } from "./components/PlanaAdminHeader";
export { PStatusBar } from "./components/PlanaStatusBar";
export { PConnectionStatus } from "./components/PlanaConnectionStatus";
export type { PBackendStatus } from "./components/PlanaConnectionStatus";
export { PPageHeader } from "./components/PlanaPageHeader";
export { PAdminTablePage } from "./components/PlanaAdminTablePage";
export type { PTableColumn } from "./components/PlanaAdminTablePage";
export { PLocalePicker } from "./components/PlanaLocalePicker";
export { PNavSidebar } from "./components/PlanaNavSidebar";
export { PFooter } from "./components/PlanaFooter";
export { PClock } from "./components/PlanaClock";
export { PSystemTray } from "./components/PlanaSystemTray";
export { PCookieConsent } from "./components/PlanaCookieConsent";
export { PICPBadge } from "./components/PlanaICPBadge";
export { PCountdownDigit } from "./components/PlanaCountdownDigit";
export type { LocaleOption } from "./components/PlanaAdminHeader";
export type { PlanaConnectionInfo } from "./components/PlanaConnectionInfo";
export { useConnectionInfo } from "./components/PlanaConnectionInfo";
export type { ConnectionStateInput } from "./components/PlanaConnectionInfo";
export { provideActionBar, useActionBar } from "./composables/useActionBar";
export type { ActionBarRenderer } from "./composables/useActionBar";
export { setProbeClient, useConnectionProbe } from "./composables/useConnectionProbe";
export type { ProbeResult } from "./composables/useConnectionProbe";
export { useEngineHealth } from "./composables/useEngineHealth";
export type { EngineHealth, EngineNetworkInfo } from "./composables/useEngineHealth";
export { usePageTitle, useRouteTitle } from "./composables/usePageTitle";
export {
  formatTokenCount,
  formatRelativeTime,
  formatUptime,
  formatMediaTime,
  formatNumber,
  formatBytes,
  formatPriceUsd,
} from "./utils/format";

// ── UUID helpers (secure-context safe; upstreamed from shittim-chest P5#A A4) ─
export { uuid, uuidv7, uuidv5 } from "./utils/uuid";
export { useAttachments } from "./composables/useAttachments";
export type { UploadedFile } from "./composables/useAttachments";

export { createAuthGuard } from "./composables/createAuthGuard";
export type { AuthGuardOptions } from "./composables/createAuthGuard";
export { renderAvatarTemplate } from "./composables/useAvatarTemplate";
export { createAdminCrudStore } from "./composables/createAdminCrudStore";
export type { AdminCrudApi } from "./composables/createAdminCrudStore";
export { PReadOnlyResourceView } from "./components/PlanaReadOnlyResourceView";
export type { PReadOnlyResource } from "./components/PlanaReadOnlyResourceView";
export { useSendShortcut } from "./composables/useSendShortcut";
export type { SendShortcutMode } from "./composables/useSendShortcut";
export { createLocaleOptions, loadLocaleMessages } from "./utils/localeOptions";

export { detectLocale } from "./utils/locale";
export { resolveErrorMessage, parseServerErrorBody, serverErrorToI18nKey } from "./utils/errors";
export type { TranslateFn } from "./utils/errors";
export { useAsyncData } from "./composables/useAsyncData";
export type { UseAsyncDataReturn } from "./composables/useAsyncData";
export { defineMockRpcData, hasMockRpcData, getMockRpcData, isDemoHost, setMockHost } from "./composables/mockRpcData";
export type { MockRpcRegistry, MockRpcValue } from "./composables/mockRpcData";
export { createRpcCall } from "./composables/useRpcCall";
export type { RpcTransport, RpcCallOptions } from "./composables/useRpcCall";
export { useConfirm } from "./composables/useConfirm";
export { useCaptchaGate } from "./composables/useCaptchaGate";
export type { PCaptchaDescriptor } from "./composables/useCaptchaGate";
export { useClipboardWithToast } from "./composables/useClipboard";
export { useRunWithLoading } from "./composables/useRunWithLoading";
export { fetchChallenge, negotiateNonce } from "./utils/powNonce";
export type { ChallengeDescriptor } from "./utils/powNonce";
export { solvePow, solvePowSync, verifyPow, leadingZeroBits } from "./utils/pow";
export type { PowChallenge, PowSolution } from "./utils/pow";
export { default as PAuthSubmitButton } from "./components/PlanaAuthSubmitButton";
export type { AuthSubmitContext } from "./components/PlanaAuthSubmitButton";

// ── Chat / LLM component kit ──────────────────────────────────────
export { PRichInput } from "./components/PlanaRichInput";
export { PVoiceInputPopup } from "./components/PlanaVoiceInputPopup";
export { PAttachmentModal, previewKindFor } from "./components/PlanaAttachmentModal";
export type { PAttachmentPreviewType } from "./components/PlanaAttachmentModal";
export { PChatMessage } from "./components/PlanaChatMessage";
export { PMcpToolBlock, parseMcpCallText, buildJsonTree, buildHighlightedLines, extractExecCode } from "./components/PlanaMcpToolBlock";
export type { PHighlightedLine, PJsonNode, PMcpToolBlockVariant, PParsedMcpCall } from "./components/PlanaMcpToolBlock";
export { PTokenUsageBadge } from "./components/PlanaTokenUsageBadge";
export { PTokenUsagePanel } from "./components/PlanaTokenUsagePanel";
export { PModelTag } from "./components/PlanaModelTag";
export {
  getModelMeta,
  registerModelCatalog,
  splitModelId,
} from "./components/PlanaModelCatalog";
export type {
  PModelCatalog,
  PModelMeta,
  PModelPricing,
} from "./components/PlanaModelCatalog";
export { PModelDownloadCard } from "./components/PlanaModelDownloadCard";
export type {
  PAttachmentDetail,
  PAttachmentItem,
  PChatRole,
  PMcpToolCall,
  PMcpToolCallStatus,
  PModelCosts,
  PModelDownloadStatus,
  PModelUsageEntry,
  PVoicePopupMode,
  PVoiceState,
} from "./components/PlanaChatTypes";

// ── Platform / auth polish kit ────────────────────────────────────
export { PCaptchaWidget } from "./components/PlanaCaptchaWidget";
export type { PCaptchaProvider } from "./components/PlanaCaptchaWidget";
export { PCaptchaModal } from "./components/PlanaCaptchaModal";
export { PProtocolModal } from "./components/PlanaProtocolModal";
export { PAboutModal } from "./components/PlanaAboutModal";
export type { PAboutLink } from "./components/PlanaAboutModal";
export { PThemeToggle } from "./components/PlanaThemeToggle";
export { PColorSchemeDialog } from "./components/PlanaColorSchemeDialog";
export { PSecretRevealModal } from "./components/PlanaSecretRevealModal";
export { PLogWindow } from "./components/PlanaLogWindow";
export type { PLogTab } from "./components/PlanaLogWindow";
export { PBreadcrumb } from "./components/PlanaBreadcrumb";
export type {
  PBreadcrumbBadge,
  PBreadcrumbItem,
  PBreadcrumbParamChip,
} from "./components/PlanaBreadcrumb";
export { default as PMinimap } from "./components/PlanaMinimap";
export { default as PLocalizedInput } from "./components/PlanaLocalizedInput";
export type { PLocalizedTitle, PLocaleOption } from "./components/PlanaLocalizedInput";
export { LOCALE_FAMILY, resolveLocalizedTitle } from "./utils/localizedTitle";

export { useAvatarUrl, type AvatarUrlUser } from "./composables/useAvatarUrl";
