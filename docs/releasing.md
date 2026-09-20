# Releasing cgrade

Keep Cargo.toml, Cargo.lock, package.json, jsr.json, pyproject.toml, and
typst.toml versions consistent. Published contents are immutable: use a new
version for corrections.

1. Update versions and CHANGELOG.md, then regenerate canonical tables with
   `bash js/@corbet-foss/cgrade/scripts/sync-assets.sh`.
2. Commit the complete release source. Package checks build each archive
   once, install it in an isolated consumer, and export its SHA-256 digest,
   source commit, and check receipt. Reuse successful results only when their
   source, dependencies, tools, and relevant environment match.
3. Tag the verified source and create the matching GitHub release. Attach the
   crate, npm tarball, Python wheel and source distribution, Typst archive,
   SOURCE_COMMIT, SHA256SUMS, and receipts. Keep existing published tags intact.
4. Import those exact source archives, receipts and packages into a publication
   bundle using the shared import contract of the release tooling.
   Review its producing commits, existing tag target and SHA-256. Attach the
   bundle to the release for hosted execution, or retain it on the release worker.
5. Run the manual release step in `status` mode, then select intended missing
   registry uploads. Cargo, npm and PyPI use the verified archives; JSR uses the
   bundle's explicitly recorded TypeScript transformations and file inventory.
   Publish sibling dependencies before cletter and resolve its committed locks
   against those registry versions. Local path overrides are integration evidence.
6. Retain the immutable publication journals and downloaded-byte verification,
   run the selected published-consumer checks, and state status per registry.
   A successful upload response or version collision is insufficient proof.

The npm registry serves npm, pnpm, Yarn, Bun, Deno's npm imports, and browser
CDNs. A successful Linux check does not prove native Windows/macOS behavior.
Minimum-runtime checks are separate from current-runtime checks. Record any
unavailable runtime or registry explicitly instead of marking it verified.

The Typst archive installs into an `@local` package namespace. Typst Universe
requires a separate eligible submission; see [installation](installation.md).
