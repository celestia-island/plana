import { describe, expect, it } from "vitest";
import { previewKindFor } from "../src/components/PlanaAttachmentModal";

describe("previewKindFor", () => {
  it("infers image/video/audio from the MIME type", () => {
    expect(previewKindFor({ type: "image/png" })).toBe("image");
    expect(previewKindFor({ type: "image/svg+xml" })).toBe("image");
    expect(previewKindFor({ type: "video/mp4" })).toBe("video");
    expect(previewKindFor({ type: "audio/mpeg" })).toBe("audio");
  });
  it("falls back to other for unknown or empty types", () => {
    expect(previewKindFor({ type: "application/pdf" })).toBe("other");
    expect(previewKindFor({ type: "" })).toBe("other");
    expect(previewKindFor(null)).toBe("other");
    expect(previewKindFor(undefined)).toBe("other");
  });
  it("lets an explicit hint win over the MIME type", () => {
    expect(previewKindFor({ type: "text/plain" }, "video")).toBe("video");
    expect(previewKindFor({ type: "image/png" }, "other")).toBe("other");
  });
});
