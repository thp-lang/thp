import { createReadStream } from "node:fs";
import { stat } from "node:fs/promises";
import { createServer, type ServerResponse } from "node:http";
import path from "node:path";
import { promisify } from "node:util";
import { execFile } from "node:child_process";
import chokidar from "chokidar";
import { build } from "./build.js";

const root = path.resolve(import.meta.dirname, "..");
const port = Number(process.env.PORT ?? 4173);
const clients = new Set<ServerResponse>();
const runFile = promisify(execFile);
let building = Promise.resolve();

async function rebuild(): Promise<void> {
  building = building.then(async () => {
    try {
      const model = await build({ root });
      await runFile(
        path.join(root, "node_modules/.bin/pagefind"),
        ["--site", "dist"],
        {
          cwd: root,
        },
      );
      console.log(`Rebuilt ${model.pages.length} pages`);
      for (const client of clients) client.write("data: reload\n\n");
    } catch (error) {
      console.error(error);
    }
  });
  await building;
}

await rebuild();
chokidar
  .watch(["content", "src", "site.yaml"], { cwd: root, ignoreInitial: true })
  .on("all", () => void rebuild());

const mime: Record<string, string> = {
  ".css": "text/css",
  ".html": "text/html",
  ".js": "text/javascript",
  ".svg": "image/svg+xml",
};
createServer(async (request, response) => {
  if (request.url === "/__reload") {
    response.writeHead(200, {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
    });
    clients.add(response);
    request.on("close", () => clients.delete(response));
    return;
  }
  const pathname = decodeURIComponent((request.url ?? "/").split("?")[0]!);
  const relative = pathname.endsWith("/") ? `${pathname}index.html` : pathname;
  const target = path.resolve(root, "dist", `.${relative}`);
  if (!target.startsWith(path.join(root, "dist"))) {
    response.writeHead(403).end("Forbidden");
    return;
  }
  try {
    const info = await stat(target);
    if (!info.isFile()) throw new Error("Not a file");
    response.writeHead(200, {
      "Content-Type": mime[path.extname(target)] ?? "application/octet-stream",
    });
    createReadStream(target).pipe(response);
  } catch {
    response.writeHead(404).end("Not found");
  }
}).listen(port, () => console.log(`THP docs: http://localhost:${port}/`));
