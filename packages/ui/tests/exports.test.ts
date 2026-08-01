/**
 * Consumer-import verification: every public export a downstream app would
 * use must be reachable from the package root, with the expected types.
 */
import { describe, expect, expectTypeOf, it } from "vitest";
import {
  PAdminTablePage,
  PConnectionStatus,
  PPageHeader,
  PStatusBar,
  formatMediaTime,
  formatNumber,
  formatRelativeTime,
  formatTokenCount,
  formatUptime,
  setProbeClient,
  useConnectionInfo,
  useConnectionProbe,
  useEngineHealth,
  type EngineHealth,
  type PBackendStatus,
  type PTableColumn,
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
