import { describe, expect, it } from "vitest";
import { mount, type VueWrapper } from "@vue/test-utils";
import { nextTick } from "vue";
import { PRichInput } from "../src/components/PlanaRichInput";

type RichInputWrapper = VueWrapper<InstanceType<typeof PRichInput>>;

/** Type into the textarea and mirror update:modelValue like a v-model parent. */
async function typeInto(wrapper: RichInputWrapper, text: string) {
  const textarea = wrapper.find("textarea");
  await textarea.setValue(text);
  const emitted = wrapper.emitted("update:modelValue");
  const last = emitted?.[emitted.length - 1]?.[0];
  if (typeof last === "string") {
    await wrapper.setProps({ modelValue: last });
    await nextTick();
  }
}

async function pressEnter(wrapper: RichInputWrapper, opts: { shiftKey?: boolean } = {}) {
  await wrapper.find("textarea").trigger("keydown", { key: "Enter", ...opts });
}

describe("PRichInput", () => {
  it("emits submit with trimmed text on Enter", async () => {
    const wrapper = mount(PRichInput, { attachTo: document.body });
    await typeInto(wrapper, "  hello world  ");
    await pressEnter(wrapper);
    expect(wrapper.emitted("submit")?.[0]).toEqual(["hello world", undefined]);
    wrapper.unmount();
  });

  it("does not submit empty input on Enter", async () => {
    const wrapper = mount(PRichInput, { attachTo: document.body });
    await typeInto(wrapper, "   ");
    await pressEnter(wrapper);
    expect(wrapper.emitted("submit")).toBeUndefined();
    wrapper.unmount();
  });

  it("inserts a newline on Shift+Enter without submitting", async () => {
    const wrapper = mount(PRichInput, { attachTo: document.body });
    await typeInto(wrapper, "line one");
    await pressEnter(wrapper, { shiftKey: true });
    expect(wrapper.emitted("submit")).toBeUndefined();
    wrapper.unmount();
  });

  it("does not submit when loading", async () => {
    const wrapper = mount(PRichInput, { props: { loading: true }, attachTo: document.body });
    await typeInto(wrapper, "hello");
    await pressEnter(wrapper);
    expect(wrapper.emitted("submit")).toBeUndefined();
    wrapper.unmount();
  });

  it("honors the canSubmit override", async () => {
    const wrapper = mount(PRichInput, { props: { canSubmit: true }, attachTo: document.body });
    await pressEnter(wrapper);
    expect(wrapper.emitted("submit")?.[0][0]).toBe("");
    wrapper.unmount();
  });

  it("does not submit when sendOnEnter is false", async () => {
    const wrapper = mount(PRichInput, { props: { sendOnEnter: false }, attachTo: document.body });
    await typeInto(wrapper, "hello");
    await pressEnter(wrapper);
    expect(wrapper.emitted("submit")).toBeUndefined();
    wrapper.unmount();
  });

  it("disables the send button on empty input and enables it after typing", async () => {
    const wrapper = mount(PRichInput, { attachTo: document.body });
    const send = wrapper.find("button[aria-label='Send']");
    expect(send.attributes("disabled")).toBeDefined();
    await typeInto(wrapper, "hello");
    expect(send.attributes("disabled")).toBeUndefined();
    wrapper.unmount();
  });

  it("hides the send button with hideSend", () => {
    const wrapper = mount(PRichInput, { props: { hideSend: true } });
    expect(wrapper.find("button[aria-label='Send']").exists()).toBe(false);
    wrapper.unmount();
  });

  it("emits pickAttachment when the paperclip is clicked", async () => {
    const wrapper = mount(PRichInput, { attachTo: document.body });
    await wrapper.find(".s-rich-input-tool-btn").trigger("click");
    expect(wrapper.emitted("pickAttachment")).toBeTruthy();
    wrapper.unmount();
  });

  it("emits removeAttachment for strip rows", async () => {
    const attachments = [
      { id: "a1", name: "spec.pdf", type: "application/pdf", size: 1024 },
    ];
    const wrapper = mount(PRichInput, {
      props: { attachments },
      attachTo: document.body,
    });
    await wrapper.find(".s-rich-input-attachment-remove").trigger("click");
    expect(wrapper.emitted("removeAttachment")?.[0]).toEqual(["a1"]);
    wrapper.unmount();
  });

  it("emits voiceToggle on mic click and renders the popup when voice is set", async () => {
    const wrapper = mount(PRichInput, {
      props: { voice: { open: true, mode: "listening" } },
      attachTo: document.body,
    });
    await wrapper.find(".s-rich-input-mic").trigger("click");
    expect(wrapper.emitted("voiceToggle")).toBeTruthy();
    // HPopover teleports its panel to <body>.
    expect(document.body.querySelector(".s-voice-popup-body")).not.toBeNull();
    wrapper.unmount();
  });

  it("emits dropFiles on drop", async () => {
    const wrapper = mount(PRichInput, { attachTo: document.body });
    const files = [new File(["a"], "a.txt")];
    await wrapper.find(".s-rich-input-body").trigger("drop", {
      dataTransfer: { files },
    });
    const emitted = wrapper.emitted("dropFiles");
    expect(emitted).toBeTruthy();
    expect(emitted?.[0][0]).toHaveLength(1);
    wrapper.unmount();
  });
});
