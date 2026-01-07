// Description: Create per-folder zip bundles for blankstop (app + src-tauri [+ docs]).

import { execSync, spawnSync } from 'node:child_process';
import { promises as fs } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const scriptRoot = path.dirname(scriptPath);
const repoRoot = path.resolve(scriptRoot, '..', '..');
const outputRoot = path.join(scriptRoot, 'output');

function formatTimestamp(date) {
  const pad = (value) => String(value).padStart(2, '0');
  return `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}_${pad(
    date.getHours(),
  )}${pad(date.getMinutes())}${pad(date.getSeconds())}`;
}

function getGitShortSha() {
  try {
    return execSync('git rev-parse --short HEAD', {
      cwd: repoRoot,
      stdio: ['ignore', 'pipe', 'ignore'],
    })
      .toString()
      .trim();
  } catch {
    return '';
  }
}

function isWsl() {
  if (process.platform !== 'linux') {
    return false;
  }
  if (process.env.WSL_DISTRO_NAME || process.env.WSL_INTEROP) {
    return true;
  }
  try {
    const version = execSync('cat /proc/version', { stdio: ['ignore', 'pipe', 'ignore'] })
      .toString()
      .toLowerCase();
    return version.includes('microsoft');
  } catch {
    return false;
  }
}

function tryCommand(cmd, args) {
  const result = spawnSync(cmd, args, { stdio: 'ignore' });
  return result.status === 0;
}

function resolvePowerShell() {
  const candidates = ['pwsh', 'powershell', 'powershell.exe'];
  for (const candidate of candidates) {
    if (tryCommand(candidate, ['-NoProfile', '-Command', '$PSVersionTable.PSVersion'])) {
      return candidate;
    }
  }
  return null;
}

function toWindowsPath(wslPath) {
  return execSync(`wslpath -w "${wslPath.replace(/"/g, '\\"')}"`, {
    stdio: ['ignore', 'pipe', 'ignore'],
  })
    .toString()
    .trim();
}

async function ensureDir(dirPath) {
  await fs.mkdir(dirPath, { recursive: true });
}

async function listGitFiles(paths) {
  const args = ['ls-files', '-z', '--', ...paths];
  try {
    const output = execSync(`git ${args.map((arg) => `"${arg}"`).join(' ')}`, {
      cwd: repoRoot,
      stdio: ['ignore', 'pipe', 'ignore'],
      encoding: 'utf8',
    });
    return output
      .split('\0')
      .map((entry) => entry.trim())
      .filter(Boolean);
  } catch {
    return [];
  }
}

async function listFilesFallback(rootDir, relativeRoot = '') {
  const entries = await fs.readdir(rootDir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(rootDir, entry.name);
    const relativePath = path.join(relativeRoot, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await listFilesFallback(fullPath, relativePath)));
    } else if (entry.isFile()) {
      files.push(relativePath.replace(/\\/g, '/'));
    }
  }
  return files;
}

async function getFilesForBundle(roots) {
  const gitFiles = await listGitFiles(roots);
  if (gitFiles.length > 0) {
    return gitFiles;
  }
  const fallback = [];
  for (const root of roots) {
    const rootPath = path.join(repoRoot, root);
    try {
      const files = await listFilesFallback(rootPath);
      fallback.push(...files.map((file) => path.join(root, file).replace(/\\/g, '/')));
    } catch {
      // ignore missing roots
    }
  }
  return fallback;
}

async function stageFiles(files, stagingRoot) {
  for (const file of files) {
    const source = path.join(repoRoot, file);
    const destination = path.join(stagingRoot, file);
    await ensureDir(path.dirname(destination));
    await fs.copyFile(source, destination);
  }
}

async function listFiles(rootDir) {
  const entries = await fs.readdir(rootDir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(rootDir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await listFiles(fullPath)));
    } else if (entry.isFile()) {
      files.push(fullPath);
    }
  }
  return files.sort();
}

async function writeManifest(stagingRoot, bundleName, sha, roots) {
  const generatedAt = new Date().toISOString();
  const shaLabel = sha || 'unavailable';
  const manifestPath = path.join(stagingRoot, '_MANIFEST.md');
  const header = [
    '# blankstop bundle manifest',
    '',
    `Bundle: ${bundleName}`,
    `Generated: ${generatedAt}`,
    `Git SHA: ${shaLabel}`,
    `Roots: ${roots.join(', ')}`,
    '',
    'Files:',
  ];
  await fs.writeFile(manifestPath, `${header.join('\n')}\n`, 'utf8');

  const files = await listFiles(stagingRoot);
  const lines = files.map((filePath) => {
    const relative = path.relative(stagingRoot, filePath).replace(/\\/g, '/');
    return `- ${relative}`;
  });
  await fs.appendFile(manifestPath, `${lines.join('\n')}\n`, 'utf8');
}

function compressArchive(stagingRoot, zipPath) {
  const psCommand = resolvePowerShell();
  if (!psCommand) {
    console.error('PowerShell not found. Install PowerShell 7 (pwsh) or ensure powershell.exe is available.');
    throw new Error('PowerShell not found.');
  }

  const needsWindowsPath = isWsl() && psCommand.toLowerCase().endsWith('.exe');
  const sourceRoot = needsWindowsPath ? toWindowsPath(stagingRoot) : stagingRoot;
  const destination = needsWindowsPath ? toWindowsPath(zipPath) : zipPath;
  const escapeLiteral = (value) => `'${String(value).replace(/'/g, "''")}'`;

  const command = [
    'Set-StrictMode -Version Latest;',
    "$ErrorActionPreference = 'Stop';",
    'Add-Type -AssemblyName System.IO.Compression;',
    'Add-Type -AssemblyName System.IO.Compression.FileSystem;',
    `$src = ${escapeLiteral(sourceRoot)};`,
    `$dest = ${escapeLiteral(destination)};`,
    'if (Test-Path -LiteralPath $dest) { Remove-Item -LiteralPath $dest -Force; }',
    '$zip = [System.IO.Compression.ZipFile]::Open($dest, [System.IO.Compression.ZipArchiveMode]::Create);',
    'try {',
    '  $files = Get-ChildItem -LiteralPath $src -Recurse -File;',
    "  $srcTrim = $src -replace '[\\\\/]+$','';",
    '  $prefix = $srcTrim + [IO.Path]::DirectorySeparatorChar;',
    '  foreach ($file in $files) {',
    '    $full = $file.FullName;',
    '    if ($full.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {',
    '      $relative = $full.Substring($prefix.Length);',
    '    } else {',
    '      $relative = $file.Name;',
    '    }',
    "    $entryName = $relative -replace '\\\\','/';",
    '    [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(' +
      '$zip, $file.FullName, $entryName, [System.IO.Compression.CompressionLevel]::Optimal' +
      ') | Out-Null;',
    '  }',
    '} finally {',
    '  $zip.Dispose();',
    '}',
  ].join(' ');

  const args = ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', command];
  const result = spawnSync(psCommand, args, { stdio: 'inherit' });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`Zip failed with exit code ${result.status}.`);
  }
}

async function removePath(targetPath) {
  try {
    await fs.rm(targetPath, { recursive: true, force: true });
  } catch {
    // ignore cleanup errors
  }
}

async function buildBundle({ bundleName, roots, timestamp, sha }) {
  const shaSuffix = sha ? `_${sha}` : '';
  const zipName = `blankstop_${bundleName}_${timestamp}${shaSuffix}.zip`;
  const zipPath = path.join(outputRoot, zipName);

  await ensureDir(outputRoot);
  await removePath(zipPath);

  const stagingRoot = path.join(outputRoot, `_staging_${bundleName}_${Date.now()}`);
  try {
    await ensureDir(stagingRoot);
    const files = await getFilesForBundle(roots);
    if (files.length === 0) {
      throw new Error(`No files found for bundle ${bundleName}.`);
    }
    await stageFiles(files, stagingRoot);
    await writeManifest(stagingRoot, bundleName, sha, roots);
    compressArchive(stagingRoot, zipPath);
  } finally {
    await removePath(stagingRoot);
  }

  const zipStats = await fs.stat(zipPath);
  if (zipStats.size === 0) {
    await removePath(zipPath);
    throw new Error(`Zip creation failed for ${bundleName}; output was empty.`);
  }

  return { bundleName, zipName, zipPath, zipBytes: zipStats.size };
}

async function writeLatestCopy(result) {
  const latestName = `blankstop_${result.bundleName}_latest.zip`;
  const latestPath = path.join(outputRoot, latestName);
  await fs.copyFile(result.zipPath, latestPath);
  return { latestName, latestPath };
}

async function writeIndex(results, sha) {
  const indexPath = path.join(outputRoot, 'blankstop_bundles_index.md');
  const lines = [
    '# blankstop bundle index',
    '',
    `Generated: ${new Date().toISOString()}`,
    `Git SHA: ${sha || 'unavailable'}`,
    '',
    'Bundles:',
  ];
  for (const result of results) {
    const mb = (bytes) => (bytes / (1024 * 1024)).toFixed(2);
    lines.push(`- ${result.bundleName}: ${result.zipName} (${mb(result.zipBytes)} MB)`);
  }
  lines.push('');
  lines.push('Latest:');
  for (const result of results) {
    lines.push(`- blankstop_${result.bundleName}_latest.zip`);
  }
  lines.push('');
  lines.push('Notes:');
  lines.push('- Bundles are assembled from git-tracked files (git ls-files).');
  await fs.writeFile(indexPath, `${lines.join('\n')}\n`, 'utf8');
}

const timestamp = formatTimestamp(new Date());
const sha = getGitShortSha();
const args = process.argv.slice(2);
const latestOnly = args.includes('--latest-only');
const includeDocs = args.includes('--docs') || args.includes('--docs-only');

const bundles = [
  { bundleName: 'app', roots: ['app'] },
  { bundleName: 'src_tauri', roots: ['src-tauri'] },
];

if (includeDocs) {
  bundles.push({ bundleName: 'docs', roots: ['docs'] });
}

const results = [];
for (const bundle of bundles) {
  const result = await buildBundle({ ...bundle, timestamp, sha });
  const latest = await writeLatestCopy(result);
  if (latestOnly) {
    const latestStats = await fs.stat(latest.latestPath);
    results.push({
      ...result,
      zipPath: latest.latestPath,
      zipName: latest.latestName,
      zipBytes: latestStats.size,
    });
    await removePath(result.zipPath);
  } else {
    results.push(result);
  }
}

await writeIndex(results, sha);
console.log(`Created ${results.length} bundle(s) in ${outputRoot}`);
