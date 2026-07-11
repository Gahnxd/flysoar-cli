#!/usr/bin/env node
// Copies the repo-root install.sh into public/ so Vercel serves it at
// https://flysoar-cli.vercel.app/install.sh (kept in sync automatically on every build).

const fs = require("fs");
const path = require("path");

const source = path.join(__dirname, "..", "..", "install.sh");
const destDir = path.join(__dirname, "..", "public");
const dest = path.join(destDir, "install.sh");

if (!fs.existsSync(source)) {
  console.warn(`[copy-install-script] Source not found at ${source}, skipping.`);
  process.exit(0);
}

fs.mkdirSync(destDir, { recursive: true });
fs.copyFileSync(source, dest);
console.log(`[copy-install-script] Copied install.sh -> ${dest}`);
