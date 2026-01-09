#!/usr/bin/env node
/**
 * Generate NSIS installer branding bitmaps from the app icon.
 * Usage: node scripts/generate_installer_bitmaps.mjs
 *
 * Outputs:
 *   src-tauri/windows/installer_header.bmp (150x57)
 *   src-tauri/windows/installer_sidebar.bmp (164x314)
 */

import sharp from 'sharp';
import { mkdir, writeFile } from 'fs/promises';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, '..');

const ICON_SOURCE = join(ROOT, 'src-tauri/icons/icon.png');
const OUT_DIR = join(ROOT, 'src-tauri/windows');

// NSIS recommended dimensions
const HEADER = { width: 150, height: 57 };
const SIDEBAR = { width: 164, height: 314 };

// Dark gradient colors (matches app theme)
const BG_TOP = { r: 30, g: 35, b: 45 };
const BG_BOTTOM = { r: 15, g: 18, b: 25 };

/**
 * Create a vertical gradient background.
 * @param {number} width
 * @param {number} height
 * @returns {Buffer} Raw RGBA pixel buffer
 */
function createGradient(width, height) {
  const buf = Buffer.alloc(width * height * 4);
  for (let y = 0; y < height; y++) {
    const t = y / (height - 1);
    const r = Math.round(BG_TOP.r + (BG_BOTTOM.r - BG_TOP.r) * t);
    const g = Math.round(BG_TOP.g + (BG_BOTTOM.g - BG_TOP.g) * t);
    const b = Math.round(BG_TOP.b + (BG_BOTTOM.b - BG_TOP.b) * t);
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      buf[i] = r;
      buf[i + 1] = g;
      buf[i + 2] = b;
      buf[i + 3] = 255;
    }
  }
  return buf;
}

/**
 * Write a 24-bit BMP file (no alpha, NSIS doesn't support it).
 * @param {string} path Output file path
 * @param {Buffer} rgbaBuffer RGBA pixel data (top-down)
 * @param {number} width
 * @param {number} height
 */
async function writeBmp(path, rgbaBuffer, width, height) {
  // BMP stores rows bottom-up, padded to 4-byte boundary
  const rowStride = Math.ceil((width * 3) / 4) * 4;
  const pixelDataSize = rowStride * height;
  const fileSize = 54 + pixelDataSize; // 14 (file header) + 40 (DIB header) + pixels

  const bmp = Buffer.alloc(fileSize);

  // File header (14 bytes)
  bmp.write('BM', 0); // signature
  bmp.writeUInt32LE(fileSize, 2); // file size
  bmp.writeUInt32LE(0, 6); // reserved
  bmp.writeUInt32LE(54, 10); // pixel data offset

  // DIB header - BITMAPINFOHEADER (40 bytes)
  bmp.writeUInt32LE(40, 14); // header size
  bmp.writeInt32LE(width, 18); // width
  bmp.writeInt32LE(height, 22); // height (positive = bottom-up)
  bmp.writeUInt16LE(1, 26); // color planes
  bmp.writeUInt16LE(24, 28); // bits per pixel
  bmp.writeUInt32LE(0, 30); // compression (none)
  bmp.writeUInt32LE(pixelDataSize, 34); // image size
  bmp.writeInt32LE(2835, 38); // horizontal resolution (72 DPI)
  bmp.writeInt32LE(2835, 42); // vertical resolution
  bmp.writeUInt32LE(0, 46); // colors in palette
  bmp.writeUInt32LE(0, 50); // important colors

  // Pixel data (bottom-up, BGR)
  for (let y = 0; y < height; y++) {
    const srcY = height - 1 - y; // flip vertically
    for (let x = 0; x < width; x++) {
      const srcIdx = (srcY * width + x) * 4;
      const dstIdx = 54 + y * rowStride + x * 3;
      bmp[dstIdx] = rgbaBuffer[srcIdx + 2]; // B
      bmp[dstIdx + 1] = rgbaBuffer[srcIdx + 1]; // G
      bmp[dstIdx + 2] = rgbaBuffer[srcIdx]; // R
    }
  }

  await writeFile(path, bmp);
}

/**
 * Generate the header image (150x57) with icon on the right.
 */
async function generateHeader() {
  const { width, height } = HEADER;
  const iconSize = Math.floor(height * 0.7);

  // Create gradient background
  const bgBuffer = createGradient(width, height);
  const background = sharp(bgBuffer, { raw: { width, height, channels: 4 } });

  // Load and resize icon
  const icon = await sharp(ICON_SOURCE)
    .resize(iconSize, iconSize, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .toBuffer();

  // Composite icon onto background (right side, vertically centered)
  const composite = await background
    .composite([
      {
        input: icon,
        left: width - iconSize - 10,
        top: Math.floor((height - iconSize) / 2),
      },
    ])
    .raw()
    .toBuffer();

  const outPath = join(OUT_DIR, 'installer_header.bmp');
  await writeBmp(outPath, composite, width, height);
  console.log(`  ${outPath} (${width}x${height})`);
}

/**
 * Generate the sidebar image (164x314) with icon centered.
 */
async function generateSidebar() {
  const { width, height } = SIDEBAR;
  const iconSize = Math.floor(width * 0.7);

  // Create gradient background
  const bgBuffer = createGradient(width, height);
  const background = sharp(bgBuffer, { raw: { width, height, channels: 4 } });

  // Load and resize icon
  const icon = await sharp(ICON_SOURCE)
    .resize(iconSize, iconSize, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .toBuffer();

  // Composite icon onto background (centered horizontally, upper third)
  const composite = await background
    .composite([
      {
        input: icon,
        left: Math.floor((width - iconSize) / 2),
        top: Math.floor(height * 0.15),
      },
    ])
    .raw()
    .toBuffer();

  const outPath = join(OUT_DIR, 'installer_sidebar.bmp');
  await writeBmp(outPath, composite, width, height);
  console.log(`  ${outPath} (${width}x${height})`);
}

async function main() {
  console.log('Generating NSIS installer bitmaps...\n');
  console.log(`Source: ${ICON_SOURCE}`);
  console.log(`Output: ${OUT_DIR}\n`);

  await mkdir(OUT_DIR, { recursive: true });

  await generateHeader();
  await generateSidebar();

  console.log('\nDone!');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
