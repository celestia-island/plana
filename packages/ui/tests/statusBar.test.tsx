import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { PStatusBar } from "../src/components/PlanaStatusBar";
import type { PlanaConnectionInfo } from "../src/components/PlanaConnectionInfo";

function makeInfo(overrides: Partial<PlanaConnectionInfo> = {}): PlanaConnectionInfo {
  return {
    state: "reconnecting",
    tier: "ws",
    quality: "good",
    latencyMs: null,
    isLocalhost: false,
    region: "US",
    retryCount: 2,
    maxRetries: 5,
    asn: null,
    attemptNumber: 2,
    countdown: 0,
    ...overrides,
  };
}

async function mountOpenPopover(info: PlanaConnectionInfo, status: "connected" | "reconnecting" | "disconnected" | "connecting" = "reconnecting") {
  const wrapper = mount(PStatusBar, {
    props: { connectionStatus: status, connectionInfo: info },
    attachTo: document.body,
  });
  await wrapper.find(".s-status-bar-tag").trigger("mouseenter");
  await nextTick();
  return wrapper;
}

describe("PStatusBar", () => {
  it("honors info.maxRetries in the retrying label", async () => {
    const wrapper = await mountOpenPopover(makeInfo({ attemptNumber: 2, maxRetries: 5 }));
    expect(document.body.textContent).toContain("2 / 5");
    expect(document.body.textContent).not.toContain("2 / 3");
    wrapper.unmount();
  });

  it("honors info.quality for the popover quality icon while connected", async () => {
    const wrapper = await mountOpenPopover(makeInfo({ state: "connected", quality: "poor" }), "connected");
    expect(document.body.querySelector("svg.lucide-wifi-off")).not.toBeNull();
    wrapper.unmount();
  });

  it("honors an excellent info.quality even while disconnected", async () => {
    const wrapper = await mountOpenPopover(makeInfo({ state: "disconnected", quality: "excellent" }), "disconnected");
    expect(document.body.querySelector("svg.lucide-wifi-off")).toBeNull();
    expect(document.body.querySelector("svg.lucide-wifi")).not.toBeNull();
    wrapper.unmount();
  });

  it("renders the ASN in the network row when present", async () => {
    const withAsn = await mountOpenPopover(makeInfo({ asn: 4134 }));
    expect(document.body.textContent).toContain("AS4134");
    withAsn.unmount();

    const withoutAsn = await mountOpenPopover(makeInfo({ asn: null }));
    expect(document.body.textContent).not.toContain("AS4134");
    withoutAsn.unmount();
  });

  it("renders measured latency in the popover header", async () => {
    const wrapper = await mountOpenPopover(makeInfo({ state: "connected", quality: "good", latencyMs: 42 }), "connected");
    expect(document.body.textContent).toContain("42 ms");
    wrapper.unmount();
  });
});
