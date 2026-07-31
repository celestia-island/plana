// plana's npm packages are normally installed as part of a host workspace
// (e.g. shittim-chest) where sibling checkouts such as hikari are registered
// as workspace packages. When installing standalone from this repo, pnpm
// cannot resolve the `workspace:*` protocol for packages that live outside
// this checkout, so rewrite those specs to `link:` paths at read time.
// The committed package.json files keep their original `workspace:*` specs.
const LINKED_WORKSPACE_DEPS = {
  // Package name -> link target, relative to the dependent package's dir.
  "@celestia-island/hikari": "../../../hikari/packages/vue",
};

function readPackage(pkg) {
  for (const field of [
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
  ]) {
    const deps = pkg[field];
    if (!deps) continue;
    for (const [name, spec] of Object.entries(deps)) {
      const target = LINKED_WORKSPACE_DEPS[name];
      if (target && typeof spec === "string" && spec.startsWith("workspace:")) {
        deps[name] = `link:${target}`;
      }
    }
  }
  return pkg;
}

module.exports = { hooks: { readPackage } };
