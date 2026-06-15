import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..", "..");

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+([.-].+)?$/.test(version)) {
  console.error(`usage: set-version.mjs <version> (got: ${version ?? "none"})`);
  process.exit(1);
}

function patchJson(relPath, mutate) {
  const path = resolve(root, relPath);
  const json = JSON.parse(readFileSync(path, "utf8"));
  mutate(json);
  writeFileSync(path, JSON.stringify(json, null, 2) + "\n");
  console.log(`set version in ${relPath} -> ${version}`);
}

patchJson("package.json", (j) => {
  j.version = version;
});

patchJson("src-tauri/tauri.conf.json", (j) => {
  j.version = version;
});

const cargoPath = resolve(root, "src-tauri/Cargo.toml");
const cargo = readFileSync(cargoPath, "utf8");
let seenPackage = false;
const patchedCargo = cargo
  .split("\n")
  .map((line) => {
    if (line.trim() === "[package]") {
      seenPackage = true;
      return line;
    }
    if (seenPackage && /^version\s*=\s*".*"/.test(line.trim())) {
      seenPackage = false;
      return `version = "${version}"`;
    }
    return line;
  })
  .join("\n");
writeFileSync(cargoPath, patchedCargo);
console.log(`set version in src-tauri/Cargo.toml -> ${version}`);
