import { createHash } from "node:crypto";
import {
  readdirSync,
  readFileSync,
  writeFileSync,
  statSync,
  mkdirSync
} from "node:fs";
import { join, extname } from "node:path";

const [, , artifactsDir, version, outDir] = process.argv;
if (!artifactsDir || !version || !outDir) {
  console.error("usage: manifest.mjs <artifactsDir> <version> <outDir>");
  process.exit(1);
}

const EXT_KEY = {
  ".deb": "linux-deb",
  ".rpm": "linux-rpm",
  ".exe": "win-nsis",
  ".msi": "win-msi",
  ".pkg": "mac-pkg",
  ".zip": "mac-zip"
};

function walk(dir) {
  const out = [];
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) {
      out.push(...walk(full));
    } else {
      out.push(full);
    }
  }
  return out;
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

const files = walk(artifactsDir);
const artifacts = {};
const checksumLines = [];

for (const file of files) {
  const ext = extname(file).toLowerCase();
  const name = file.split("/").pop();
  const key = name.endsWith(".app.zip") ? "mac-zip" : EXT_KEY[ext];
  if (!key) continue;
  artifacts[key] = name;
  checksumLines.push(`${sha256(file)}  ${name}`);
}

if (Object.keys(artifacts).length === 0) {
  console.error(`no recognized artifacts found in ${artifactsDir}`);
  process.exit(1);
}

const manifest = {
  version,
  releaseDate: new Date().toISOString(),
  artifacts
};

mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, "manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
writeFileSync(join(outDir, "sha256Checksum.txt"), checksumLines.join("\n") + "\n");

console.log("manifest.json:");
console.log(JSON.stringify(manifest, null, 2));
console.log("sha256Checksum.txt:");
console.log(checksumLines.join("\n"));
