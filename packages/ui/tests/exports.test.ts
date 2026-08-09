/**
 * Consumer-import verification: every public export a downstream app would
 * use must be reachable from the package root, with the expected types.
 */
import { describe, expect, expectTypeOf, it } from "vitest";
import {
  PAboutModal,
  PAdminTablePage,
  PAttachmentModal,
  PBreadcrumb,
  PCaptchaModal,
  PCaptchaWidget,
  PChatMessage,
  PColorSchemeDialog,
  PConnectionStatus,
  PLogWindow,
  PToolBlock,
  PModelDownloadCard,
  PModelTag,
  PPageHeader,
  PProtocolModal,
  PRichInput,
  PSecretRevealModal,
  PStatusBar,
  PThemeToggle,
  PTokenUsageBadge,
  PTokenUsagePanel,
  PVoiceInputPopup,
  formatBytes,
  formatMediaTime,
  formatNumber,
  formatPriceUsd,
  formatRelativeTime,
  formatTokenCount,
  formatUptime,
  getModelMeta,
  parseToolCallText,
  registerModelCatalog,
  setProbeClient,
  splitModelId,
  useConnectionInfo,
  useConnectionProbe,
  useEngineHealth,
  type EngineHealth,
  type PAttachmentItem,
  type PBackendStatus,
  type PBreadcrumbBadge,
  type PBreadcrumbItem,
  type PBreadcrumbParamChip,
  type PCaptchaProvider,
  type PToolCall,
  type PModelMeta,
  type PLogTab,
  type PTableColumn,
  type PVoiceState,
  type ProbeResult,
} from "../src/index";

describe("package exports", () => {
  it("exposes the new components", () => {
    expect(PConnectionStatus).toBeDefined();
    expect(PPageHeader).toBeDefined();
    expect(PAdminTablePage).toBeDefined();
    expect(PStatusBar).toBeDefined();
  });

  it("exposes the composables", () => {
    expect(useEngineHealth).toBeTypeOf("function");
    expect(useConnectionProbe).toBeTypeOf("function");
    expect(useConnectionInfo).toBeTypeOf("function");
    expect(setProbeClient).toBeTypeOf("function");
  });

  it("exposes the format helpers", () => {
    expect(formatTokenCount).toBeTypeOf("function");
    expect(formatRelativeTime).toBeTypeOf("function");
    expect(formatUptime).toBeTypeOf("function");
    expect(formatMediaTime).toBeTypeOf("function");
    expect(formatNumber).toBeTypeOf("function");
    expect(formatBytes).toBeTypeOf("function");
    expect(formatPriceUsd).toBeTypeOf("function");
  });

  it("exposes the chat/LLM kit components and utils", () => {
    expect(PRichInput).toBeDefined();
    expect(PVoiceInputPopup).toBeDefined();
    expect(PAttachmentModal).toBeDefined();
    expect(PChatMessage).toBeDefined();
    expect(PToolBlock).toBeDefined();
    expect(PTokenUsageBadge).toBeDefined();
    expect(PTokenUsagePanel).toBeDefined();
    expect(PModelTag).toBeDefined();
    expect(PModelDownloadCard).toBeDefined();
    expect(splitModelId).toBeTypeOf("function");
    expect(getModelMeta).toBeTypeOf("function");
    expect(registerModelCatalog).toBeTypeOf("function");
    expect(parseToolCallText).toBeTypeOf("function");
  });

  it("exposes the chat/LLM kit public types", () => {
    const tool: PToolCall = { toolName: "web_search", status: "done", callText: "" };
    const att: PAttachmentItem = { id: "a", name: "x.txt", type: "text/plain", size: 1 };
    const voice: PVoiceState = { open: false, mode: "listening" };
    const meta: PModelMeta = { contextWindow: 8192 };
    expect(tool.status).toBe("done");
    expect(att.name).toBe("x.txt");
    expect(voice.mode).toBe("listening");
    expect(meta.contextWindow).toBe(8192);
    expectTypeOf<PToolCall["status"]>().toEqualTypeOf<"pending" | "running" | "done" | "error">();
  });

  it("exposes the platform/auth polish kit components and types", () => {
    expect(PCaptchaWidget).toBeDefined();
    expect(PCaptchaModal).toBeDefined();
    expect(PProtocolModal).toBeDefined();
    expect(PAboutModal).toBeDefined();
    expect(PThemeToggle).toBeDefined();
    expect(PColorSchemeDialog).toBeDefined();
    expect(PSecretRevealModal).toBeDefined();
    expect(PLogWindow).toBeDefined();
    expect(PBreadcrumb).toBeDefined();

    const provider: PCaptchaProvider = "recaptcha";
    const tab: PLogTab = { key: "server", title: "Server", lines: ["ok"] };
    const item: PBreadcrumbItem = { label: "Home", to: "/", active: false };
    const badge: PBreadcrumbBadge = { id: "b1", text: "OK", variant: "success" };
    const param: PBreadcrumbParamChip = { id: "p1", label: "engine", value: "demo" };
    expect(provider).toBe("recaptcha");
    expect(tab.lines).toContain("ok");
    expect(item.active).toBe(false);
    expect(badge.variant).toBe("success");
    expect(param.value).toBe("demo");
    expectTypeOf<PCaptchaProvider>().toEqualTypeOf<"turnstile" | "recaptcha">();
  });

  it("exposes the new public types", () => {
    const column: PTableColumn = { key: "name", title: "Name", align: "left" };
    const backend: PBackendStatus = { label: "Engine", state: "ok" };
    const health: EngineHealth = {
      engineVersion: "1.2.3",
      engineBuildHash: "abc123",
      network: { region: "CN", asn: 4134, transport: "ws" },
    };
    expect(column.key).toBe("name");
    expect(backend.state).toBe("ok");
    expect(health.network?.region).toBe("CN");

    // ProbeResult keeps the deprecated mirrors for one release.
    expectTypeOf<ProbeResult["transportTier"]>().toEqualTypeOf<string>();
    expectTypeOf<ProbeResult["tier"]>().toEqualTypeOf<string>();
    expectTypeOf<ProbeResult["attemptNumber"]>().toEqualTypeOf<number>();
    expectTypeOf<ProbeResult["retryCount"]>().toEqualTypeOf<number>();
    expectTypeOf<ProbeResult["latencyMs"]>().toEqualTypeOf<number | null>();
  });
});
