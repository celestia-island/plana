import { describe, expect, it } from "vitest";

import { useAsyncData } from "../src/composables/useAsyncData";

describe("useAsyncData", () => {
  it("does not auto-fetch when immediate is false", () => {
    const { data, loading } = useAsyncData(
      async () => 42,
      { immediate: false },
    );
    expect(data.value).toBeNull();
    expect(loading.value).toBe(false);
  });

  it("refresh() populates data on success", async () => {
    const { data, loading, refresh } = useAsyncData(
      async () => "hello",
      { immediate: false },
    );
    await refresh();
    expect(data.value).toBe("hello");
    expect(loading.value).toBe(false);
  });

  it("refresh() sets error on failure", async () => {
    const { data, error, refresh } = useAsyncData(
      async () => { throw new Error("fail"); },
      { immediate: false },
    );
    await refresh();
    expect(data.value).toBeNull();
    expect(error.value).toBeInstanceOf(Error);
    expect(error.value!.message).toBe("fail");
  });

  it("sets loading during fetch", async () => {
    let resolveFetch!: (v: number) => void;
    const fetcher = () => new Promise<number>((r) => { resolveFetch = r; });
    const { loading, refresh } = useAsyncData(fetcher, { immediate: false });
    const p = refresh();
    expect(loading.value).toBe(true);
    resolveFetch(1);
    await p;
    expect(loading.value).toBe(false);
  });

  it("wraps non-Error rejects in Error", async () => {
    const { error, refresh } = useAsyncData(
      async () => { throw "string error"; },
      { immediate: false },
    );
    await refresh();
    expect(error.value).toBeInstanceOf(Error);
    expect(error.value!.message).toBe("string error");
  });
});
