// Type-only package: the entire surface is TypeScript declarations served
// from index.d.ts. This placeholder exists so resolvers that follow `main`
// (Node require, Jest module maps) land on a loadable file instead of the
// raw binding source; it deliberately exports nothing at runtime.
module.exports = {};
