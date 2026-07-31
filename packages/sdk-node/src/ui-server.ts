import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export type UiServerOptions = {
  port: number;
  adminUrl: string;
  staticDir?: string;
  host?: string;
};

const MIME: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".ico": "image/x-icon",
};

function defaultStaticDir(): string {
  const here = path.dirname(fileURLToPath(import.meta.url));
  return path.join(here, "..", "ui-static");
}

export function startUiServer(opts: UiServerOptions): Promise<{
  url: string;
  stop: () => void;
}> {
  const staticDir = opts.staticDir ?? defaultStaticDir();
  const host = opts.host ?? "127.0.0.1";

  if (!fs.existsSync(staticDir)) {
    return Promise.reject(
      new Error(
        `UI static assets not found at ${staticDir}. Run scripts/bundle-ui.sh before building the SDK.`
      )
    );
  }

  const server = http.createServer((req, res) => {
    const url = req.url?.split("?")[0] ?? "/";

    if (url === "/config.js") {
      const body = `window.__FEATHER_CONFIG__ = ${JSON.stringify({
        adminUrl: opts.adminUrl,
      })};`;
      res.writeHead(200, { "Content-Type": "text/javascript; charset=utf-8" });
      res.end(body);
      return;
    }

    let filePath = path.join(staticDir, url === "/" ? "index.html" : url);
    if (!filePath.startsWith(staticDir)) {
      res.writeHead(403).end();
      return;
    }

    if (!fs.existsSync(filePath) || fs.statSync(filePath).isDirectory()) {
      filePath = path.join(staticDir, "index.html");
    }

    const ext = path.extname(filePath);
    res.writeHead(200, {
      "Content-Type": MIME[ext] ?? "application/octet-stream",
    });
    fs.createReadStream(filePath).pipe(res);
  });

  return new Promise((resolve, reject) => {
    server.on("error", reject);
    server.listen(opts.port, host, () => {
      const url = `http://${host}:${opts.port}`;
      resolve({
        url,
        stop: () => server.close(),
      });
    });
  });
}
