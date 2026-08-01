/**
 * Consumer-import verification: every public export a downstream app would
 * use must be reachable from the package root, with the expected types.
 */
import { describe, expect, expectTypeOf, it } from "vitest";
import {
  PAdminTablePage,
  PAttachmentModal,
  PChatMessage,
  PConnectionStatus,
  PMcpToolBlock,
  PModelDownloadCard,
  PModelTag,
  PPageHeader,
  PRichInput,
  PStatusBar,
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
  parseMcpCallText,
  registerModelCatalog,
  setProbeClient,
  splitModelId,
  useConnectionInfo,
  useConnectionProbe,
  useEngineHealth,
  type EngineHealth,
  type PAttachmentItem,
  type PBackendStatus,
  type PMcpToolCall,
  type PModelMeta,
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
    expect(PMcpToolBlock).toBeDefined();
    expect(PTokenUsageBadge).toBeDefined();
    expect(PTokenUsagePanel).toBeDefined();
    expect(PModelTag).toBeDefined();
    expect(PModelDownloadCard).toBeDefined();
    expect(splitModelId).toBeTypeOf("function");
    expect(getModelMeta).toBeTypeOf("function");
    expect(registerModelCatalog).toBeTypeOf("function");
    expect(parseMcpCallText).toBeTypeOf("function");
  });

  it("exposes the chat/LLM kit public types", () => {
    const tool: PMcpToolCall = { toolName: "web_search", status: "done", callText: "" };
    const att: PAttachmentItem = { id: "a", name: "x.txt", type: "text/plain", size: 1 };
    const voice: PVoiceState = { open: false, mode: "listening" };
    const meta: PModelMeta = { contextWindow: 8192 };
    expect(tool.status).toBe("done");
    expect(att.name).toBe("x.txt");
    expect(voice.mode).toBe("listening");
    expect(meta.contextWindow).toBe(8192);
    expectTypeOf<PMcpToolCall["status"]>().toEqualTypeOf<"pending" | "running" | "done" | "error">();
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
