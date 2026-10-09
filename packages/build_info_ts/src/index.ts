/**
 * Family build identity — the TS twin of the Rust `plana_build_info`
 * crate, for build-time webui tooling (Vite plugins and the like).
 *
 * User direction 2026-10-08: the numeric patch counter is retired; the
 * branch and the exact commit ARE the identity, rendered as
 * `<base> <branch>::<hash7>` (e.g. `0.1 master::d423747`). This module
 * resolves the `<branch>::<hash7>` identity token of a source tree so a
 * webui build stamp means the same thing as every engine's version line.
 *
 * Resolution order mirrors the Rust crate and `celestia-devtools
 * version-string`:
 * 1. the `celestia-devtools` facility, when on the build PATH;
 * 2. direct git — branch via `rev-parse --abbrev-ref HEAD` (detached
 *    worktrees resolved through local heads → origin remote heads →
 *    symbolic remote HEAD → `"detached"`), hash via
 *    `rev-parse --short=7 HEAD`;
 * 3. no git metadata → `undefined`, so callers keep their own
 *    deterministic fallback instead of a fabricated token.
 *
 * Node context only (spawns git / devtools); not for browser bundles.
 */
import { execFileSync } from "node:child_process";

/** Shape of the identity token: `<branch>::<hash7>` (hash 7–40 hex). */
export const IDENTITY_RE = /^(\S+)::([0-9a-f]{7,40})$/i;

/** Validate and normalize one `<branch>::<hash>` candidate line. */
export function parseIdentityToken(line: string): string | undefined {
  const token = line
    .split(/\s+/)
    .reverse()
    .find((part) => part.includes("::"));
  const match = token?.match(IDENTITY_RE);
  return match ? `${match[1]}::${match[2].toLowerCase()}` : undefined;
}

/** Whether a value is a bare 7–40 char **lowercase** hex commit hash
 * (git prints lowercase; canonicalize before calling). */
export function isShortHash(value: string): boolean {
  return /^[0-9a-f]{7,40}$/i.test(value) && value === value.toLowerCase();
}

function run(
  command: string,
  args: string[],
  cwd: string,
  timeoutMs: number,
): string | undefined {
  try {
    const out = execFileSync(command, args, {
      cwd,
      encoding: "utf-8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: timeoutMs,
    });
    const value = out.trim();
    return value === "" ? undefined : value;
  } catch {
    return undefined;
  }
}

/** Branch name of `root`'s HEAD, detached worktrees included. */
function branchOf(root: string): string | undefined {
  const branch = run("git", ["rev-parse", "--abbrev-ref", "HEAD"], root, 10000);
  if (branch !== undefined && branch !== "HEAD") return branch;

  // Detached ladder: local heads, then origin's remote heads (a freshly
  // fetched squash sha may live only on the remote), then the symbolic
  // remote HEAD. `name-rev` prints the literal `undefined` when its ref
  // set cannot name the commit — that must not leak into the identity.
  for (const refs of ["refs/heads/*", "refs/remotes/origin/*"]) {
    const named = run(
      "git",
      ["name-rev", "--name-only", `--refs=${refs}`, "HEAD"],
      root,
      10000,
    );
    if (named && named !== "undefined" && named !== "remotes/origin/HEAD") {
      // Git refnames cannot contain '~', so this drops any depth suffix
      // (`master~2`); remote names keep their last self.
      return named.split("~")[0].replace(/^remotes\/origin\//, "");
    }
  }
  const sym = run(
    "git",
    ["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    root,
    10000,
  );
  if (sym) return sym.split("/").pop();
  return "detached";
}

/**
 * Resolve the `<branch>::<hash7>` build identity of the git worktree
 * containing `root` (any directory inside the tree works), or `undefined`
 * when neither the devtools facility nor git can answer — callers keep
 * their own deterministic fallback for non-git builds (source tarballs).
 */
export function resolveBuildIdentity(root: string): string | undefined {
  const facility = run(
    "celestia-devtools",
    ["version-string", "--dir", root],
    root,
    15000,
  );
  const fromFacility = facility ? parseIdentityToken(facility) : undefined;
  if (fromFacility) return fromFacility;

  const branch = branchOf(root);
  const hash = run("git", ["rev-parse", "--short=7", "HEAD"], root, 10000);
  if (!branch || !hash || !isShortHash(hash)) return undefined;
  return `${branch}::${hash.toLowerCase()}`;
}
