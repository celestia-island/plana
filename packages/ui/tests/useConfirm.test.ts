import { describe, expect, it } from "vitest";

import { useConfirm } from "../src/composables/useConfirm";

describe("useConfirm", () => {
  it("starts with hidden state", () => {
    const { visible, title, message } = useConfirm();
    expect(visible.value).toBe(false);
    expect(title.value).toBe("Confirm");
    expect(message.value).toBe("");
  });

  it("confirm() shows dialog and returns promise", () => {
    const { visible, title, message, confirm } = useConfirm();
    confirm("Delete?", "Are you sure?");
    expect(visible.value).toBe(true);
    expect(title.value).toBe("Delete?");
    expect(message.value).toBe("Are you sure?");
  });

  it("accept() resolves promise with true", async () => {
    const { confirm, accept } = useConfirm();
    const promise = confirm("Title", "Msg");
    accept();
    const result = await promise;
    expect(result).toBe(true);
  });

  it("cancel() resolves promise with false", async () => {
    const { confirm, cancel } = useConfirm();
    const promise = confirm("Title", "Msg");
    cancel();
    const result = await promise;
    expect(result).toBe(false);
  });

  it("accept() hides dialog", () => {
    const { visible, confirm, accept } = useConfirm();
    confirm("T", "M");
    accept();
    expect(visible.value).toBe(false);
  });

  it("cancel() hides dialog", () => {
    const { visible, confirm, cancel } = useConfirm();
    confirm("T", "M");
    cancel();
    expect(visible.value).toBe(false);
  });
});
