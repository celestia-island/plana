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
