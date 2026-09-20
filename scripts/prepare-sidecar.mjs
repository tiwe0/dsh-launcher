import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  chmodSync,
  createWriteStream,
  existsSync,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { Readable } from "node:stream";
import { finished } from "node:stream/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";

const NVM_VERSION = "0.40.7";
const NVM_SHA256 = "d2fb84dba9914b02cd69b97df35dfca8695b8f22df6128667034d85b69b52d57";
const NVM_URL = `https://github.com/nvm-sh/nvm/archive/refs/tags/v${NVM_VERSION}.tar.gz`;
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const tauriDir = path.join(root, "src-tauri");
const resourceDir = path.join(tauriDir, "resources");
const archivePath = path.join(resourceDir, `nvm-v${NVM_VERSION}.tar.gz`);

function sha256(file) {
  return createHash("sha256").update(readFileSync(file)).digest("hex");
}

async function prepareNvm() {
  mkdirSync(resourceDir, { recursive: true });
  if (existsSync(archivePath) && sha256(archivePath) === NVM_SHA256) return;
  const temporary = `${archivePath}.download`;
  rmSync(temporary, { force: true });
  const response = await fetch(NVM_URL, { redirect: "follow" });
  if (!response.ok || !response.body) {
    throw new Error(`Unable to download NVM ${NVM_VERSION}: HTTP ${response.status}`);
  }
  await finished(Readable.fromWeb(response.body).pipe(createWriteStream(temporary)));
  const actual = sha256(temporary);
  if (actual !== NVM_SHA256) {
    rmSync(temporary, { force: true });
    throw new Error(`NVM archive checksum mismatch: expected ${NVM_SHA256}, received ${actual}`);
  }
  renameSync(temporary, archivePath);
}

function rustHost() {
  const output = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const match = output.match(/^host:\s*(\S+)$/m);
  if (!match) throw new Error("Unable to determine Rust host target");
  return match[1];
}

function requestedTarget() {
  const flagIndex = process.argv.indexOf("--target");
  if (flagIndex >= 0 && process.argv[flagIndex + 1]) return process.argv[flagIndex + 1];
  return process.env.TAURI_ENV_TARGET_TRIPLE || rustHost();
}

function buildSidecar() {
  const target = requestedTarget();
  execFileSync(
    "cargo",
    [
      "build",
      "--release",
      "--target",
      target,
      "--manifest-path",
      path.join(tauriDir, "sidecar", "Cargo.toml"),
    ],
    { cwd: root, stdio: "inherit" },
  );
  const extension = target.includes("windows") ? ".exe" : "";
  const source = path.join(tauriDir, "sidecar", "target", target, "release", `dsh-runtime${extension}`);
  const destination = path.join(tauriDir, "sidecar", `dsh-runtime-${target}${extension}`);
  const temporary = `${destination}.tmp`;
  rmSync(temporary, { force: true });
  writeFileSync(temporary, readFileSync(source));
  chmodSync(temporary, 0o755);
  renameSync(temporary, destination);
  console.log(`Prepared ${path.relative(root, destination)}`);
}

await prepareNvm();
buildSidecar();
