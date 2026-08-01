import { describe, expect, it } from "vitest";
import { ref } from "vue";
import { useConnectionInfo, type ConnectionStateInput } from "../src/components/PlanaConnectionInfo";

function makeInfo(
  state: ConnectionStateInput,
  opts: {
    tier?: string;
    retryCount?: number;
    maxRetries?: number;
    attemptNumber?: number;
    countdown?: number;
    latencyMs?: number | null;
  } = {},
) {
  const { connectionInfo } = useConnectionInfo(
    ref(state),
    opts.tier !== undefined ? ref(opts.tier) : undefined,
    opts.retryCount !== undefined ? ref(opts.retryCount) : undefined,
    opts.maxRetries !== undefined ? ref(opts.maxRetries) : undefined,
    opts.attemptNumber !== undefined ? ref(opts.attemptNumber) : undefined,
    opts.countdown !== undefined ? ref(opts.countdown) : undefined,
    opts.latencyMs !== undefined ? ref(opts.latencyMs) : undefined,
  );
  return connectionInfo.value;
}

describe("useConnectionInfo", () => {
  it("derives quality from tier when connected", () => {
    expect(makeInfo("connected", { tier: "local" }).quality).toBe("excellent");
    expect(makeInfo("connected", { tier: "ws" }).quality).toBe("good");
    expect(makeInfo("connected", { tier: "sse" }).quality).toBe("fair");
  });

  it("reports quality unknown when not connected", () => {
    expect(makeInfo("disconnected", { tier: "ws" }).quality).toBe("unknown");
    expect(makeInfo("reconnecting", { tier: "ws" }).quality).toBe("unknown");
  });

  it("passes maxRetries through (PStatusBar renders it instead of a hardcoded 3)", () => {
    expect(makeInfo("reconnecting", { maxRetries: 5 }).maxRetries).toBe(5);
    expect(makeInfo("reconnecting").maxRetries).toBe(3);
  });

  it("maps input states to the status-bar state triplet", () => {
    expect(makeInfo("connected").state).toBe("connected");
    expect(makeInfo("connecting").state).toBe("reconnecting");
    expect(makeInfo("reconnecting").state).toBe("reconnecting");
    expect(makeInfo("failed").state).toBe("disconnected");
    expect(makeInfo("disconnected").state).toBe("disconnected");
  });

  it("accepts a latency ref and surfaces it in the info", () => {
    expect(makeInfo("connected", { tier: "ws", latencyMs: 42 }).latencyMs).toBe(42);
    expect(makeInfo("connected", { tier: "ws" }).latencyMs).toBeNull();
  });

  it("tracks reactive latency updates", () => {
    const latency = ref<number | null>(10);
    const { connectionInfo } = useConnectionInfo(
      ref("connected" as ConnectionStateInput),
      ref("ws"),
      undefined,
      undefined,
      undefined,
      undefined,
      latency,
    );
    expect(connectionInfo.value.latencyMs).toBe(10);
    latency.value = 25;
    expect(connectionInfo.value.latencyMs).toBe(25);
    latency.value = null;
    expect(connectionInfo.value.latencyMs).toBeNull();
  });
});
