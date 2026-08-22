import path from "node:path";
import { build } from "./build.js";

const command = process.argv[2];
if (command !== "build") throw new Error(`Unknown command "${command ?? ""}"`);
const root = path.resolve(import.meta.dirname, "..");
const model = await build({ root });
console.log(
  `Rendered ${model.pages.length} pages at base path ${model.basePath}`,
);
