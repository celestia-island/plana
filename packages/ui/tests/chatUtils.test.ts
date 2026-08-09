import { describe, expect, it } from "vitest";
import { formatBytes, formatPriceUsd } from "../src/utils/format";
import {
  getModelMeta,
  registerModelCatalog,
  splitModelId,
} from "../src/components/PlanaModelCatalog";
import { parseToolCallText } from "../src/components/PlanaToolBlock";

describe("formatBytes", () => {
  it("formats bytes verbatim", () => {
    expect(formatBytes(0)).toBe("0B");
    expect(formatBytes(512)).toBe("512B");
  });
  it("formats kilobytes", () => {
    expect(formatBytes(1024)).toBe("1.0KB");
    expect(formatBytes(1536)).toBe("1.5KB");
  });
  it("formats megabytes and gigabytes", () => {
    expect(formatBytes(2_621_440)).toBe("2.5MB");
    expect(formatBytes(3_221_225_472)).toBe("3.0GB");
  });
  it("clamps non-finite and negative input to 0B", () => {
    expect(formatBytes(NaN)).toBe("0B");
    expect(formatBytes(-100)).toBe("0B");
  });
});

describe("formatPriceUsd", () => {
  it("formats cents-scale prices", () => {
    expect(formatPriceUsd(0.1)).toBe("$0.10");
    expect(formatPriceUsd(0.125)).toBe("$0.13");
  });
  it("formats dollar-scale prices with one decimal", () => {
    expect(formatPriceUsd(1.5)).toBe("$1.5");
    expect(formatPriceUsd(15)).toBe("$15.0");
  });
  it("rounds large prices to whole dollars", () => {
    expect(formatPriceUsd(120)).toBe("$120");
    expect(formatPriceUsd(2500.7)).toBe("$2501");
  });
  it("clamps negative and non-finite input to $0.00", () => {
    expect(formatPriceUsd(-5)).toBe("$0.00");
    expect(formatPriceUsd(NaN)).toBe("$0.00");
  });
  it("honors a custom currency", () => {
    expect(formatPriceUsd(2, "€")).toBe("€2.0");
  });
});

describe("splitModelId", () => {
  it("splits base and tag at the last #", () => {
    expect(splitModelId("deepseek-v4-flash#3")).toEqual({ base: "deepseek-v4-flash", tag: "3" });
  });
  it("returns an empty tag when no # is present", () => {
    expect(splitModelId("gpt-5.5")).toEqual({ base: "gpt-5.5", tag: "" });
  });
  it("handles # inside the base name", () => {
    expect(splitModelId("a#b#c")).toEqual({ base: "a#b", tag: "c" });
  });
});

describe("getModelMeta / registerModelCatalog", () => {
  it("looks up built-in entries with the tag stripped", () => {
    const meta = getModelMeta("deepseek-v4-flash#1");
    expect(meta).toBeDefined();
    expect(meta?.contextWindow).toBe(64_000);
    expect(meta?.tools).toBe(true);
  });
  it("returns undefined for unknown models", () => {
    expect(getModelMeta("totally-unknown")).toBeUndefined();
  });
  it("merges service-registered entries over the built-in catalog", () => {
    registerModelCatalog({ "my-model": { contextWindow: 4096, maxOutput: 1024 } });
    const meta = getModelMeta("my-model");
    expect(meta?.contextWindow).toBe(4096);
    // Built-in entries still resolve.
    expect(getModelMeta("gpt-5.5")?.vision).toBe(true);
  });
  it("prefers an explicit per-call catalog", () => {
    registerModelCatalog({ "shared": { contextWindow: 10_000 } });
    const meta = getModelMeta("shared", { shared: { contextWindow: 99 } });
    expect(meta?.contextWindow).toBe(99);
  });
});

describe("parseToolCallText", () => {
  it("parses a quoted-name args call", () => {
    const parsed = parseToolCallText('"web_search", {"query": "celestia", "limit": 5}');
    expect(parsed).not.toBeNull();
    expect(parsed?.toolName).toBe("web_search");
    expect(parsed?.argsObj).toEqual({ query: "celestia", limit: 5 });
  });
  it("keeps unparseable args without throwing", () => {
    const parsed = parseToolCallText('"browse", {broken json}');
    expect(parsed?.toolName).toBe("browse");
    expect(parsed?.argsObj).toBeNull();
  });
  it("returns null for non-matching text", () => {
    expect(parseToolCallText("plain text")).toBeNull();
    expect(parseToolCallText("")).toBeNull();
  });
});
