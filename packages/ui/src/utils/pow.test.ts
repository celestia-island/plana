import { describe, expect, it } from "vitest";
import { leadingZeroBits, solvePow, solvePowSync, verifyPow } from "./pow";

describe("pow", () => {
  it("leadingZeroBits counts correctly", () => {
    expect(leadingZeroBits(new Uint8Array([0, 0, 1, 0]))).toBe(16);
    expect(leadingZeroBits(new Uint8Array([0x80, 0, 0]))).toBe(0);
    expect(leadingZeroBits(new Uint8Array([0x01, 0, 0]))).toBe(7);
  });

  it("solvePow finds a counter meeting the difficulty", async () => {
    const counter = await solvePow({ seed: "test-seed", bits: 8 });
    expect(await verifyPow({ seed: "test-seed", bits: 8 }, counter)).toBe(true);
  });

  it("the sync solver matches the subtle path (same wire contract)", async () => {
    const counter = solvePowSync("test-seed", 8);
    expect(await verifyPow({ seed: "test-seed", bits: 8 }, counter)).toBe(true);
    const c1 = solvePowSync("fixed-seed", 12);
    const c2 = solvePowSync("fixed-seed", 12);
    expect(c1).toBe(c2);
  });

  it("the hash layout is deterministic (wire contract)", async () => {
    const c1 = await solvePow({ seed: "fixed-seed", bits: 12 });
    const c2 = await solvePow({ seed: "fixed-seed", bits: 12 });
    expect(c1).toBe(c2);
    expect(await verifyPow({ seed: "fixed-seed", bits: 12 }, c1)).toBe(true);
  });
});
