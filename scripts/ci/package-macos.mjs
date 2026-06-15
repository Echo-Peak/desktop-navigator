import { execFileSync } from "node:child_process";
import { readdirSync, existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve, join, basename } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "..", "..");
const bundleDir = resolve(root, "src-tauri/target/release/bundle/macos");

const mode = process.argv[2];
if (mode !== "zip" && mode !== "pkg") {
  console.error("usage: package-macos.mjs <zip|pkg>");
  process.exit(1);
}

if (!existsSync(bundleDir)) {
  console.error(`bundle dir not found: ${bundleDir}`);
  process.exit(1);
}

const app = readdirSync(bundleDir).find((f) => f.endsWith(".app"));
if (!app) {
  console.error("no .app bundle found");
  process.exit(1);
}

const appPath = join(bundleDir, app);
const name = basename(app, ".app").replace(/\s+/g, "");

const pkg = JSON.parse(
  (await import("node:fs")).readFileSync(resolve(root, "package.json"), "utf8")
);
const version = pkg.version;

if (mode === "zip") {
  const out = join(bundleDir, `${name}-${version}.app.zip`);
  execFileSync(
    "ditto",
    ["-c", "-k", "--sequesterRsrc", "--keepParent", appPath, out],
    { stdio: "inherit" }
  );
  console.log(`created ${out}`);
} else {
  const out = join(bundleDir, `${name}-${version}.pkg`);
  execFileSync(
    "pkgbuild",
    [
      "--install-location",
      "/Applications",
      "--component",
      appPath,
      out
    ],
    { stdio: "inherit" }
  );
  console.log(`created ${out}`);
}
