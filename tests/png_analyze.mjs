// PNG analysis helpers for the E2E suite.
// decodePng(buf) -> { width, height, pixels } (8-bit RGBA non-interlaced PNGs).
// countOrange(w,h,pixels) -> number of warm-orange sampled pixels.
import { inflateSync } from 'node:zlib';

function decodePNG(buf) {
  let pos = 8;
  let idat = [];
  let width = 0, height = 0, colorType = 0;
  while (pos < buf.length) {
    const len = buf.readUInt32BE(pos);
    const type = buf.toString('ascii', pos + 4, pos + 8);
    if (type === 'IHDR') {
      width = buf.readUInt32BE(pos + 8);
      height = buf.readUInt32BE(pos + 12);
      colorType = buf[pos + 17];
    } else if (type === 'IDAT') {
      idat.push(buf.subarray(pos + 8, pos + 8 + len));
    } else if (type === 'IEND') break;
    pos += 12 + len;
  }
  const inflate = inflateSync(Buffer.concat(idat));
  const bpp = colorType === 6 ? 4 : 3;
  const stride = width * bpp;
  const out = Buffer.alloc(width * height * 4);
  let src = 0;
  for (let y = 0; y < height; y++) {
    const filter = inflate[src++];
    const row = y * stride;
    for (let x = 0; x < width; x++) {
      const i = row + x * bpp;
      let r, g, b, a = 255;
      if (colorType === 6) { r = inflate[src]; g = inflate[src+1]; b = inflate[src+2]; a = inflate[src+3]; src += 4; }
      else { r = inflate[src]; g = inflate[src+1]; b = inflate[src+2]; src += 3; }
      const o = (y * width + x) * 4;
      const up = y > 0 ? (y - 1) * width + x : -1;
      const left = x > 0 ? y * width + x - 1 : -1;
      const upleft = y > 0 && x > 0 ? (y - 1) * width + x - 1 : -1;
      const pa = (c, left, up, upleft) => {
        const l = left >= 0 ? out[left * 4 + c] : 0;
        const u = up >= 0 ? out[up * 4 + c] : 0;
        const ul = upleft >= 0 ? out[upleft * 4 + c] : 0;
        const p = l + u - ul;
        const pa = Math.abs(p - l), pb = Math.abs(p - u), pc = Math.abs(p - ul);
        return pa <= pb && pa <= pc ? l : pb <= pc ? u : ul;
      };
      if (filter === 1 && x > 0) { r = (r + out[left * 4]) & 255; g = (g + out[left * 4 + 1]) & 255; b = (b + out[left * 4 + 2]) & 255; }
      else if (filter === 2 && y > 0) { r = (r + out[up * 4]) & 255; g = (g + out[up * 4 + 1]) & 255; b = (b + out[up * 4 + 2]) & 255; }
      else if (filter === 3) { r = (r + Math.floor((out[left * 4] + (up >= 0 ? out[up * 4] : 0)) / 2)) & 255; g = (g + Math.floor((out[left * 4 + 1] + (up >= 0 ? out[up * 4 + 1] : 0)) / 2)) & 255; b = (b + Math.floor((out[left * 4 + 2] + (up >= 0 ? out[up * 4 + 2] : 0)) / 2)) & 255; }
      else if (filter === 4) { r = (r + pa(0, left, up, upleft)) & 255; g = (g + pa(1, left, up, upleft)) & 255; b = (b + pa(2, left, up, upleft)) & 255; }
      out[o] = r; out[o+1] = g; out[o+2] = b; out[o+3] = a;
    }
  }
  return { width, height, pixels: out };
}

export function decodePng(buf) { return decodePNG(buf); }

export function countOrange(width, height, pixels) {
  let orange = 0;
  for (let y = 0; y < height; y += 2) for (let x = 0; x < width; x += 7) {
    const i = (y * width + x) * 4;
    const r = pixels[i], g = pixels[i + 1], b = pixels[i + 2];
    if (r > 180 && g > 60 && g < 200 && b < 100) orange++;
  }
  return orange;
}