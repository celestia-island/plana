import { describe, expect, it } from "vitest";
import {
  formatMediaTime,
  formatNumber,
  formatRelativeTime,
  formatTokenCount,
  formatUptime,
} from "../src/utils/format";

describe("formatTokenCount", () => {
  it("formats small counts verbatim", () => {
    expect(formatTokenCount(0)).toBe("0");
    expect(formatTokenCount(999)).toBe("999");
  });
  it("formats thousands with k suffix", () => {
    expect(formatTokenCount(1000)).toBe("1.0k");
    expect(formatTokenCount(1234)).toBe("1.2k");
    expect(formatTokenCount(999_999)).toBe("1000.0k");
  });
  it("formats millions with M suffix", () => {
    expect(formatTokenCount(1_000_000)).toBe("1.0M");
    expect(formatTokenCount(2_500_000)).toBe("2.5M");
  });
});

describe("formatNumber", () => {
  it("formats small numbers verbatim", () => {
    expect(formatNumber(42)).toBe("42");
    expect(formatNumber(999)).toBe("999");
  });
  it("formats thousands with k suffix", () => {
    expect(formatNumber(1000)).toBe("1.0k");
    expect(formatNumber(1530)).toBe("1.5k");
  });
});

describe("formatMediaTime", () => {
  it("formats m:ss with zero-padded seconds", () => {
    expect(formatMediaTime(0)).toBe("0:00");
    expect(formatMediaTime(9)).toBe("0:09");
    expect(formatMediaTime(65)).toBe("1:05");
    expect(formatMediaTime(3599)).toBe("59:59");
    expect(formatMediaTime(3600)).toBe("60:00");
  });
  it("clamps negative and non-finite input to 0", () => {
    expect(formatMediaTime(-5)).toBe("0:00");
    expect(formatMediaTime(NaN)).toBe("0:00");
    expect(formatMediaTime(Infinity)).toBe("0:00");
  });
});

describe("formatUptime", () => {
  it("formats seconds-only under a minute", () => {
    expect(formatUptime(0)).toBe("0s");
    expect(formatUptime(45)).toBe("45s");
  });
  it("formats minutes under an hour", () => {
    expect(formatUptime(60)).toBe("1m");
    expect(formatUptime(12 * 60 + 30)).toBe("12m");
  });
  it("formats hours with remaining minutes", () => {
    expect(formatUptime(3600)).toBe("1h 0m");
    expect(formatUptime(3 * 3600 + 12 * 60)).toBe("3h 12m");
  });
  it("clamps negative and non-finite input to 0", () => {
    expect(formatUptime(-10)).toBe("0s");
    expect(formatUptime(NaN)).toBe("0s");
  });
});

describe("formatRelativeTime", () => {
  it("returns empty string for invalid input", () => {
    expect(formatRelativeTime("")).toBe("");
    expect(formatRelativeTime("not-a-date")).toBe("");
  });
  it("formats just-now for under a minute", () => {
    expect(formatRelativeTime(Date.now())).toBe("Just now");
  });
  it("formats minutes, hours and days ago", () => {
    const now = Date.now();
    expect(formatRelativeTime(now - 5 * 60_000)).toBe("5m ago");
    expect(formatRelativeTime(now - 3 * 3_600_000)).toBe("3h ago");
    expect(formatRelativeTime(now - 2 * 86_400_000)).toBe("2d ago");
  });
  it("falls back to a locale date for older timestamps", () => {
    const old = new Date(Date.now() - 30 * 86_400_000);
    expect(formatRelativeTime(old)).toBe(old.toLocaleDateString());
  });
  it("accepts ISO strings", () => {
    const iso = new Date(Date.now() - 60_000).toISOString();
    expect(formatRelativeTime(iso)).toBe("1m ago");
  });
});
