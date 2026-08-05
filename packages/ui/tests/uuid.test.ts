import { describe, expect, it } from "vitest";

import { uuidv7 } from "../src/utils/uuid";

describe("uuidv7", () => {
  it("returns a 36-char string with dashes", () => {
    const id = uuidv7();
    expect(id).toHaveLength(36);
    expect(id.split("-").length).toBe(5);
  });

  it("contains only hex chars and dashes", () => {
    const id = uuidv7();
    expect(id).toMatch(/^[0-9a-f-]+$/);
  });

  it("sets version nibble to 7", () => {
    const id = uuidv7();
    const versionChar = id[14];
    expect(versionChar).toBe("7");
  });

  it("sets variant bits to 8/9/a/b", () => {
    const id = uuidv7();
    const variantChar = id[19];
    expect(["8", "9", "a", "b"]).toContain(variantChar);
  });

  it("produces unique ids on successive calls", () => {
    const ids = new Set(Array.from({ length: 100 }, () => uuidv7()));
    expect(ids.size).toBe(100);
  });

  it("timestamp portion is monotonic across calls", () => {
    const ids = Array.from({ length: 10 }, () => uuidv7());
    for (let i = 1; i < ids.length; i++) {
      const ts1 = parseInt(ids[i - 1].slice(0, 13).replace("-", ""), 16);
      const ts2 = parseInt(ids[i].slice(0, 13).replace("-", ""), 16);
      expect(ts2 >= ts1).toBe(true);
    }
  });
});
