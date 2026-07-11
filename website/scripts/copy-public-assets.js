#!/usr/bin/env node
// Copies repo-root assets into public/ so Vercel serves them at the site root.
// Runs automatically on every dev/build via the predev/prebuild npm hooks.
//
//   install.sh  -> public/install.sh
//   SKILL.md    -> public/llms.txt   (LLM-facing documentation)
//   SKILL.md    -> public/SKILL.md   (same content, alternate name)

const fs = require("fs");
const path = require("path");

const destDir = path.join(__dirname, "..", "public");
fs.mkdirSync(destDir, { recursive: true });

const assets = [
  { source: "install.sh", dest: ["install.sh"] },
  { source: "SKILL.md", dest: ["llms.txt", "SKILL.md"] },
];

for (const { source, dest } of assets) {
  const src = path.join(__dirname, "..", "..", source);
  if (!fs.existsSync(src)) {
    console.warn(`[copy-public-assets] Source not found at ${src}, skipping.`);
    continue;
  }
  for (const name of dest) {
    const target = path.join(destDir, name);
    fs.copyFileSync(src, target);
    console.log(`[copy-public-assets] Copied ${source} -> ${target}`);
  }
}
