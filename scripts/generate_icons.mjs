#!/usr/bin/env node
/**
 * Generate all icon sizes from a source PNG.
 * Usage: node scripts/generate_icons.mjs [source.png]
 * Default source: app/assets/icon.png
 *
 * Auto-crops transparent padding so the design fills the icon space.
 */

import sharp from 'sharp';
import { mkdir, readFile, writeFile } from 'fs/promises';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');

const SOURCE = process.argv[2] || join(ROOT, 'app/assets/icon.png');
const OUT_DIR = join(ROOT, 'src-tauri/icons');

// Percentage of padding to add back after cropping (0.1 = 10% on each side)
const PADDING_RATIO = 0.08;

// All sizes needed for Tauri (Windows + macOS + Linux)
const SIZES = [
  { name: '32x32.png', size: 32 },
  { name: '128x128.png', size: 128 },
  { name: '128x128@2x.png', size: 256 },
  { name: 'icon.png', size: 512 },
  // Windows Store logos
  { name: 'Square30x30Logo.png', size: 30 },
  { name: 'Square44x44Logo.png', size: 44 },
  { name: 'Square71x71Logo.png', size: 71 },
  { name: 'Square89x89Logo.png', size: 89 },
  { name: 'Square107x107Logo.png', size: 107 },
  { name: 'Square142x142Logo.png', size: 142 },
  { name: 'Square150x150Logo.png', size: 150 },
  { name: 'Square284x284Logo.png', size: 284 },
  { name: 'Square310x310Logo.png', size: 310 },
  { name: 'StoreLogo.png', size: 50 },
];

// ICO sizes (Windows)
const ICO_SIZES = [16, 24, 32, 48, 64, 128, 256];

async function cropToContent(source) {
  // Trim transparent pixels, then add back a small padding
  const trimmed = await sharp(source).trim().toBuffer({ resolveWithObject: true });
  const { width, height } = trimmed.info;

  // Calculate padding to add back (makes the icon not touch edges)
  const maxDim = Math.max(width, height);
  const padding = Math.round(maxDim * PADDING_RATIO);
  const newSize = maxDim + padding * 2;

  // Create square canvas with the trimmed content centered
  const cropped = await sharp({
    create: {
      width: newSize,
      height: newSize,
      channels: 4,
      background: { r: 0, g: 0, b: 0, alpha: 0 },
    },
  })
    .composite([
      {
        input: trimmed.data,
        raw: { width, height, channels: 4 },
        left: Math.round((newSize - width) / 2),
        top: Math.round((newSize - height) / 2),
      },
    ])
    .png()
    .toBuffer();

  console.log(`Cropped: ${width}x${height} -> ${newSize}x${newSize} (with ${padding}px padding)\n`);
  return cropped;
}

async function generatePngs(croppedBuffer) {
  console.log(`Output: ${OUT_DIR}\n`);

  await mkdir(OUT_DIR, { recursive: true });

  for (const { name, size } of SIZES) {
    const out = join(OUT_DIR, name);
    await sharp(croppedBuffer)
      .resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
      .png()
      .toFile(out);
    console.log(`  ${name} (${size}x${size})`);
  }
}

async function generateIco(croppedBuffer) {
  // Generate ICO by embedding multiple PNG sizes
  // ICO format: header + directory entries + image data
  const images = await Promise.all(
    ICO_SIZES.map(async (size) => {
      const buf = await sharp(croppedBuffer)
        .resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
        .png()
        .toBuffer();
      return { size, data: buf };
    })
  );

  // ICO header: 2 bytes reserved, 2 bytes type (1=ico), 2 bytes count
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0); // reserved
  header.writeUInt16LE(1, 2); // type: 1 = ICO
  header.writeUInt16LE(images.length, 4); // image count

  // Directory entries: 16 bytes each
  const dirEntries = Buffer.alloc(16 * images.length);
  let offset = 6 + 16 * images.length; // after header + directory

  images.forEach((img, i) => {
    const entry = dirEntries.subarray(i * 16, (i + 1) * 16);
    entry.writeUInt8(img.size >= 256 ? 0 : img.size, 0); // width (0 = 256)
    entry.writeUInt8(img.size >= 256 ? 0 : img.size, 1); // height
    entry.writeUInt8(0, 2); // color palette
    entry.writeUInt8(0, 3); // reserved
    entry.writeUInt16LE(1, 4); // color planes
    entry.writeUInt16LE(32, 6); // bits per pixel
    entry.writeUInt32LE(img.data.length, 8); // image size
    entry.writeUInt32LE(offset, 12); // offset to image data
    offset += img.data.length;
  });

  const ico = Buffer.concat([header, dirEntries, ...images.map((img) => img.data)]);
  await writeFile(join(OUT_DIR, 'icon.ico'), ico);
  console.log(`  icon.ico (${ICO_SIZES.join(', ')})`);
}

async function generateIcns(croppedBuffer) {
  // ICNS uses specific OSType codes for each size
  const icnsTypes = [
    { type: 'icp4', size: 16 },   // 16x16
    { type: 'icp5', size: 32 },   // 32x32
    { type: 'icp6', size: 64 },   // 64x64
    { type: 'ic07', size: 128 },  // 128x128
    { type: 'ic08', size: 256 },  // 256x256
    { type: 'ic09', size: 512 },  // 512x512
    { type: 'ic10', size: 1024 }, // 1024x1024
  ];

  const images = await Promise.all(
    icnsTypes.map(async ({ type, size }) => {
      const data = await sharp(croppedBuffer)
        .resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
        .png()
        .toBuffer();
      return { type, data };
    })
  );

  // ICNS header: 4 bytes magic ('icns'), 4 bytes total file size
  // Each image: 4 bytes type, 4 bytes size (including header), data
  let totalSize = 8; // header
  for (const img of images) {
    totalSize += 8 + img.data.length;
  }

  const icns = Buffer.alloc(totalSize);
  icns.write('icns', 0); // magic
  icns.writeUInt32BE(totalSize, 4); // file size

  let offset = 8;
  for (const img of images) {
    icns.write(img.type, offset); // OSType
    icns.writeUInt32BE(8 + img.data.length, offset + 4); // chunk size
    img.data.copy(icns, offset + 8);
    offset += 8 + img.data.length;
  }

  await writeFile(join(OUT_DIR, 'icon.icns'), icns);
  console.log(`  icon.icns (${icnsTypes.map((t) => t.size).join(', ')})`);
}

async function main() {
  console.log('Generating icons...\n');
  console.log(`Source: ${SOURCE}`);

  // Load source (use as-is if already cropped, or auto-crop if needed)
  const source = await sharp(SOURCE).png().toBuffer();
  console.log('');

  await generatePngs(source);
  await generateIco(source);
  await generateIcns(source);

  console.log('\nDone!');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
