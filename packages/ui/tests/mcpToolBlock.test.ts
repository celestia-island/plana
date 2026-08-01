import { describe, expect, it } from "vitest";
import {
  buildHighlightedLines,
  buildJsonTree,
  extractExecCode,
  parseMcpCallText,
} from "../src/components/PlanaMcpToolBlock";

describe("extractExecCode", () => {
  it("extracts the code argument from a quoted-name exec call", () => {
    const code = "const x = 1;\nconsole.log(x);";
    const callText = `"run", ${JSON.stringify({ code })}`;
    expect(extractExecCode(callText)).toBe(code);
  });
  it("extracts code from a bare JSON args object", () => {
    const code = "return 42;";
    expect(extractExecCode(`{"code": ${JSON.stringify(code)}}`)).toBe(code);
  });
  it("returns null when no code argument exists", () => {
    expect(extractExecCode('"web_search", {"query": "x"}')).toBeNull();
    expect(extractExecCode("plain text")).toBeNull();
    expect(extractExecCode("")).toBeNull();
  });
});

describe("buildHighlightedLines", () => {
  it("numbers lines left-padded to the widest line number", () => {
    const lines = buildHighlightedLines("a\nb\nc", "plaintext");
    expect(lines.map(l => l.num)).toEqual(["1", "2", "3"]);
    const ten = buildHighlightedLines("a\nb\nc\nd\ne\nf\ng\nh\ni\nj", "plaintext");
    expect(ten[0].num).toBe(" 1");
  });
  it("escapes plain text when no hljs is registered", () => {
    const [line] = buildHighlightedLines("<div>&</div>", "html");
    expect(line.html).toBe("&lt;div&gt;&amp;&lt;/div&gt;");
  });
  it("keeps a single trailing newline from splitting into an empty row", () => {
    expect(buildHighlightedLines("a\n", "plaintext")).toHaveLength(2);
  });
});

describe("buildJsonTree", () => {
  it("returns null for non-container values", () => {
    expect(buildJsonTree(null)).toBeNull();
    expect(buildJsonTree("text")).toBeNull();
    expect(buildJsonTree(42)).toBeNull();
  });
  it("builds object trees with keys, values and previews", () => {
    const root = buildJsonTree({ name: "celestia", enabled: true, count: 7 });
    expect(root).not.toBeNull();
    expect(root?.key).toBeNull();
    expect(root?.childCount).toBe(3);
    expect(root?.isContainer).toBe(true);
    expect(root?.children.map(c => c.key)).toEqual(["name", "enabled", "count"]);
    expect(root?.children[0].value).toBe("celestia");
    expect(root?.children[1].value).toBe(true);
    expect(root?.preview).toContain("name: ");
  });
  it("builds array trees with index keys", () => {
    const root = buildJsonTree([10, 20]);
    expect(root?.children.map(c => c.key)).toEqual(["0", "1"]);
    expect(root?.preview).toContain("[10, 20]");
  });
  it("caps nesting at maxDepth", () => {
    const root = buildJsonTree({ a: { b: { c: { d: { e: 1 } } } } }, 2);
    const a = root?.children[0];
    const b = a?.children[0];
    // Depth 2 is the last level where children are materialized.
    expect(a?.children).toHaveLength(1);
    expect(b?.children).toHaveLength(0);
  });
  it("flags long strings and stores the full value", () => {
    const long = "x".repeat(100);
    const root = buildJsonTree({ blob: long });
    const node = root?.children[0];
    expect(node?.isLongString).toBe(true);
    expect(node?.stringValue).toBe(long);
    expect(node?.preview).toBe("");
  });
  it("assigns unique ids across the whole tree", () => {
    const root = buildJsonTree({ a: { b: 1 }, c: [2, 3] });
    const ids: number[] = [];
    function walk(n: NonNullable<typeof root>) {
      ids.push(n.id);
      for (const child of n.children) walk(child);
    }
    walk(root!);
    expect(new Set(ids).size).toBe(ids.length);
  });
});

describe("parseMcpCallText", () => {
  it("parses a quoted-name args call", () => {
    const parsed = parseMcpCallText('"web_search", {"query": "celestia", "limit": 5}');
    expect(parsed?.toolName).toBe("web_search");
    expect(parsed?.argsObj).toEqual({ query: "celestia", limit: 5 });
  });
  it("keeps unparseable args without throwing", () => {
    const parsed = parseMcpCallText('"browse", {broken json}');
    expect(parsed?.toolName).toBe("browse");
    expect(parsed?.argsObj).toBeNull();
  });
  it("returns null for non-matching text", () => {
    expect(parseMcpCallText("plain text")).toBeNull();
    expect(parseMcpCallText("")).toBeNull();
  });
});
