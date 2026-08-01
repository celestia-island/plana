/**
 * P3 platform/auth polish kit tests: captcha (placeholder mode), protocol,
 * about, theme toggle, secret reveal, log window and breadcrumb.
 */
import { describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { nextTick } from "vue";
import { initTheme } from "@celestia-island/hikari";
import {
  PAboutModal,
  PBreadcrumb,
  PCaptchaModal,
  PCaptchaWidget,
  PLogWindow,
  PProtocolModal,
  PSecretRevealModal,
  PThemeToggle,
} from "../src/index";

function stubTurnstile() {
  vi.stubGlobal("turnstile", {
    render: vi.fn(() => "widget-1"),
    reset: vi.fn(),
    remove: vi.fn(),
  });
}

function stubClipboard() {
  Object.defineProperty(navigator, "clipboard", {
    value: { writeText: vi.fn().mockResolvedValue(undefined) },
    configurable: true,
  });
}

describe("PCaptchaWidget", () => {
  it("renders a no-op placeholder without a site key", () => {
    const wrapper = mount(PCaptchaWidget, {
      props: { siteKey: "", provider: "turnstile" },
      attachTo: document.body,
    });
    expect(wrapper.find(".s-captcha-widget-placeholder").exists()).toBe(true);
    expect(document.head.querySelector("script[src*='challenges.cloudflare.com']")).toBeNull();
    wrapper.unmount();
  });

  it("enters placeholder mode when disabled", () => {
    const wrapper = mount(PCaptchaWidget, {
      props: { siteKey: "0xTEST", disabled: true },
      attachTo: document.body,
    });
    expect(wrapper.find(".s-captcha-widget-placeholder").exists()).toBe(true);
    wrapper.unmount();
  });

  it("loads the provider script when a site key is present", async () => {
    stubTurnstile();
    const wrapper = mount(PCaptchaWidget, {
      props: { siteKey: "0xTEST", provider: "turnstile", scriptUrl: "https://example.test/turnstile.js" },
      attachTo: document.body,
    });
    await nextTick();
    await nextTick();
    expect(wrapper.find(".s-captcha-widget-placeholder").exists()).toBe(false);
    expect(document.head.querySelector("script[src='https://example.test/turnstile.js']")).not.toBeNull();
    wrapper.unmount();
    vi.unstubAllGlobals();
  });
});

describe("PCaptchaModal", () => {
  it("renders prompt and widget while open", async () => {
    stubTurnstile();
    const wrapper = mount(PCaptchaModal, {
      props: { modelValue: true, siteKey: "0xTEST", scriptUrl: "https://example.test/c.js" },
      attachTo: document.body,
    });
    await nextTick();
    await nextTick();
    expect(document.body.textContent).toContain("Complete the verification below");
    expect(document.head.querySelector("script[src='https://example.test/c.js']")).not.toBeNull();
    wrapper.unmount();
    vi.unstubAllGlobals();
  });
});

describe("PProtocolModal", () => {
  it("renders markdown content and emits accept/decline", async () => {
    const wrapper = mount(PProtocolModal, {
      props: { modelValue: true, title: "Terms", content: "# Hello\n\nbody text" },
      attachTo: document.body,
    });
    await nextTick();
    await nextTick();
    const buttons = Array.from(document.body.querySelectorAll("button"));
    const accept = buttons.find((b) => b.textContent?.includes("Accept"));
    const decline = buttons.find((b) => b.textContent?.includes("Decline"));
    expect(accept).toBeDefined();
    expect(decline).toBeDefined();
    (accept as HTMLButtonElement).click();
    await nextTick();
    expect(wrapper.emitted("accept")).toHaveLength(1);
    (decline as HTMLButtonElement).click();
    await nextTick();
    expect(wrapper.emitted("decline")).toHaveLength(1);
    wrapper.unmount();
  });
});

describe("PAboutModal", () => {
  it("renders app name, version, build hashes and links", async () => {
    const wrapper = mount(PAboutModal, {
      props: {
        modelValue: true,
        appName: "Demo",
        version: "1.2.3",
        buildHash: "abcdef1234567890",
        engineVersion: "0.9.1",
        engineBuildHash: "deadbeef1234",
        links: [{ label: "GitHub", href: "https://github.com/example" }],
      },
      attachTo: document.body,
    });
    await nextTick();
    expect(document.body.textContent).toContain("Demo");
    expect(document.body.textContent).toContain("1.2.3");
    expect(document.body.textContent).toContain("abcdef123456…");
    expect(document.body.textContent).toContain("0.9.1");
    expect(document.body.querySelector('a[href="https://github.com/example"]')).not.toBeNull();
    wrapper.unmount();
  });
});

describe("PThemeToggle", () => {
  it("cycles light/dark via the main button", async () => {
    localStorage.setItem("hikari-theme-mode", "light");
    initTheme();
    const wrapper = mount(PThemeToggle, { attachTo: document.body });
    await nextTick();
    const main = wrapper.find(".s-theme-toggle-btn[data-variant='main']");
    await main.trigger("click");
    expect(localStorage.getItem("hikari-theme-mode")).toBe("dark");
    await main.trigger("click");
    expect(localStorage.getItem("hikari-theme-mode")).toBe("light");
    wrapper.unmount();
    localStorage.removeItem("hikari-theme-mode");
    localStorage.removeItem("hikari-theme");
  });

  it("opens the popover with mode options and preset themes", async () => {
    const wrapper = mount(PThemeToggle, { attachTo: document.body });
    await nextTick();
    await wrapper.find(".s-theme-toggle-btn[data-variant='arrow']").trigger("click");
    await nextTick();
    expect(document.body.textContent).toContain("Synthwave '84");
    expect(document.body.textContent).toContain("Light");
    expect(document.body.textContent).toContain("Dark");
    expect(document.body.textContent).toContain("Auto");
    wrapper.unmount();
  });
});

describe("PSecretRevealModal", () => {
  it("shows the secret and emits copied on copy", async () => {
    stubClipboard();
    const wrapper = mount(PSecretRevealModal, {
      props: { modelValue: true, label: "API Key", value: "sk-test-123" },
      attachTo: document.body,
    });
    await nextTick();
    expect(document.body.textContent).toContain("sk-test-123");
    const buttons = Array.from(document.body.querySelectorAll("button"));
    const copyBtn = buttons.find((b) => b.textContent?.includes("Copy"));
    expect(copyBtn).toBeDefined();
    (copyBtn as HTMLButtonElement).click();
    await nextTick();
    await nextTick();
    expect(wrapper.emitted("copied")).toBeDefined();
    wrapper.unmount();
  });
});

describe("PLogWindow", () => {
  const tabs = [
    { key: "server", title: "Server", lines: ["INFO [server] started", "ERROR [server] boom"] },
    { key: "tui", title: "TUI", lines: [] },
  ];

  it("renders tabs and switches active content", async () => {
    const wrapper = mount(PLogWindow, {
      props: { modelValue: true, tabs },
      attachTo: document.body,
    });
    await nextTick();
    expect(document.body.textContent).toContain("INFO [server] started");
    const tabButtons = Array.from(document.body.querySelectorAll(".s-log-tab"));
    const tuiTab = tabButtons.find((b) => b.textContent === "TUI");
    expect(tuiTab).toBeDefined();
    (tuiTab as HTMLButtonElement).click();
    await nextTick();
    expect(document.body.textContent).toContain("No log lines yet.");
    wrapper.unmount();
  });

  it("emits clearTab for the active tab", async () => {
    const wrapper = mount(PLogWindow, {
      props: { modelValue: true, tabs },
      attachTo: document.body,
    });
    await nextTick();
    const buttons = Array.from(document.body.querySelectorAll(".s-log-controls .s-log-btn"));
    (buttons[buttons.length - 1] as HTMLButtonElement).click();
    expect(wrapper.emitted("clearTab")).toEqual([["server"]]);
    wrapper.unmount();
  });

  it("emits update:paused on pause toggle", async () => {
    const wrapper = mount(PLogWindow, {
      props: { modelValue: true, tabs },
      attachTo: document.body,
    });
    await nextTick();
    const buttons = Array.from(document.body.querySelectorAll(".s-log-controls .s-log-btn"));
    (buttons[0] as HTMLButtonElement).click();
    expect(wrapper.emitted("update:paused")).toEqual([[true]]);
    wrapper.unmount();
  });
});

describe("PBreadcrumb", () => {
  it("renders items through hikari HBreadcrumb and chips", async () => {
    const wrapper = mount(PBreadcrumb, {
      props: {
        items: [
          { label: "Home", to: "/" },
          { label: "Models", to: "/models" },
          { label: "llama-3" },
        ],
        badges: [{ id: "b1", text: "OK", variant: "success", onClick: () => {} }],
        params: [{ id: "p1", label: "engine", value: "demo" }],
      },
      attachTo: document.body,
    });
    await nextTick();
    expect(wrapper.find("nav").exists()).toBe(true);
    expect(wrapper.text()).toContain("Home");
    expect(wrapper.text()).toContain("Models");
    expect(wrapper.text()).toContain("llama-3");
    expect(wrapper.find(".s-breadcrumb-badge").text()).toBe("OK");
    expect(wrapper.find(".s-breadcrumb-param").text()).toContain("engine");
    expect(wrapper.find(".s-breadcrumb-param").text()).toContain("demo");
    wrapper.unmount();
  });

  it("emits badgeClick on chip click", async () => {
    const wrapper = mount(PBreadcrumb, {
      props: {
        items: [{ label: "Home", to: "/" }],
        badges: [{ id: "b1", text: "OK", variant: "success" }],
      },
      attachTo: document.body,
    });
    await nextTick();
    await wrapper.find(".s-breadcrumb-badge").trigger("click");
    expect(wrapper.emitted("badgeClick")).toHaveLength(1);
    wrapper.unmount();
  });
});
