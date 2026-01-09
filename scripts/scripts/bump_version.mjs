// Description: Bump version across package.json, Cargo.toml, and tauri.conf.json.

import { promises as fs } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const scriptRoot = path.dirname(scriptPath);
const repoRoot = path.resolve(scriptRoot, '..', '..');

const version = process.argv[2];
if (!version) {
  console.error('Usage: node scripts/scripts/bump_version.mjs <version>');
  process.exit(1);
}

if (!/^\d+\.\d+\.\d+/.test(version)) {
  console.error(`Invalid version "${version}". Expected something like 0.1.2.`);
  process.exit(1);
}

async function updateJson(filePath, updater) {
  const raw = await fs.readFile(filePath, 'utf8');
  const data = JSON.parse(raw);
  const next = updater(data);
  await fs.writeFile(filePath, `${JSON.stringify(next, null, 2)}\n`, 'utf8');
}

async function updateCargoToml(filePath, nextVersion) {
  const raw = await fs.readFile(filePath, 'utf8');
  const lines = raw.split(/\r?\n/);
  let inPackage = false;
  let updated = false;
  const nextLines = lines.map((line) => {
    const trimmed = line.trim();
    if (/^\[.*\]$/.test(trimmed)) {
      inPackage = trimmed === '[package]';
    }
    if (inPackage && /^version\s*=/.test(trimmed)) {
      updated = true;
      return `version = "${nextVersion}"`;
    }
    return line;
  });
  if (!updated) {
    throw new Error('Cargo.toml [package] version not found.');
  }
  await fs.writeFile(filePath, `${nextLines.join('\n')}\n`, 'utf8');
}

await updateJson(path.join(repoRoot, 'package.json'), (data) => ({
  ...data,
  version,
}));

await updateJson(path.join(repoRoot, 'src-tauri', 'tauri.conf.json'), (data) => ({
  ...data,
  version,
}));

await updateCargoToml(path.join(repoRoot, 'src-tauri', 'Cargo.toml'), version);

console.log(`Updated version to ${version}.`);
