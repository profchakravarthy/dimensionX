/**
 * @fileoverview Zero-Dependency Local HTTP Static Server for dimensionX (`scripts/serve.mjs`).
 * Serves the repository root with proper ES module MIME types and path-traversal protection.
 * License: MIT
 */

import http from 'node:http';
import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');

const MIME_TYPES = Object.freeze({
  '.html': 'text/html; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.ico': 'image/x-icon'
});

export function createStaticServer(rootDir = ROOT_DIR) {
  return http.createServer(async (req, res) => {
    try {
      const reqUrl = new URL(req.url || '/', 'http://localhost');
      const rawPathname = decodeURIComponent(reqUrl.pathname);
      const relativePath = rawPathname === '/' ? '/index.html' : rawPathname;
      const resolvedPath = path.resolve(rootDir, '.' + relativePath);

      if (!resolvedPath.startsWith(rootDir)) {
        res.writeHead(403, { 'Content-Type': 'text/plain; charset=utf-8' });
        res.end('403 Forbidden');
        return;
      }

      const stat = await fs.stat(resolvedPath);
      if (stat.isDirectory()) {
        res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
        res.end('404 Not Found');
        return;
      }

      const ext = path.extname(resolvedPath).toLowerCase();
      const contentType = MIME_TYPES[ext] || 'application/octet-stream';
      const content = await fs.readFile(resolvedPath);

      res.writeHead(200, {
        'Content-Type': contentType,
        'Cache-Control': 'no-cache'
      });
      res.end(content);
    } catch {
      res.writeHead(404, { 'Content-Type': 'text/plain; charset=utf-8' });
      res.end('404 Not Found');
    }
  });
}

if (process.argv[1] && path.resolve(process.argv[1]) === __filename) {
  const port = Number(process.env.PORT) || 4173;
  const server = createStaticServer(ROOT_DIR);
  server.listen(port, '127.0.0.1', () => {
    console.log(`dimensionX observatory server listening on http://127.0.0.1:${port}`);
  });
}
