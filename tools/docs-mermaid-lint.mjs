#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";

const REPO_ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const DOCS_ROOT = path.join(REPO_ROOT, "docs", "src");
const MERMAID_BUNDLE = path.join(REPO_ROOT, "docs", "mermaid.min.js");

function walkMarkdownFiles(rootDir) {
  const out = [];
  const stack = [rootDir];
  while (stack.length > 0) {
    const dir = stack.pop();
    if (!dir) {
      continue;
    }
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        stack.push(fullPath);
      } else if (entry.isFile() && fullPath.endsWith(".md")) {
        out.push(fullPath);
      }
    }
  }
  out.sort();
  return out;
}

function extractMermaidBlocks(markdown, filePath) {
  const lines = markdown.split(/\r?\n/);
  const blocks = [];
  let inFence = false;
  let isMermaidFence = false;
  let startLine = 0;
  let buffer = [];

  for (let idx = 0; idx < lines.length; idx += 1) {
    const line = lines[idx];
    const fenceMatch = line.match(/^\s*```(.*)$/);
    if (!fenceMatch) {
      if (inFence && isMermaidFence) {
        buffer.push(line);
      }
      continue;
    }

    if (!inFence) {
      inFence = true;
      startLine = idx + 1;
      const lang = fenceMatch[1].trim().split(/\s+/, 1)[0] ?? "";
      isMermaidFence = lang === "mermaid";
      buffer = [];
      continue;
    }

    if (isMermaidFence) {
      blocks.push({
        filePath,
        startLine,
        content: buffer.join("\n"),
      });
    }
    inFence = false;
    isMermaidFence = false;
    startLine = 0;
    buffer = [];
  }

  if (inFence && isMermaidFence) {
    blocks.push({
      filePath,
      startLine,
      content: null,
    });
  }

  return blocks;
}

function loadMermaid() {
  if (!fs.existsSync(MERMAID_BUNDLE)) {
    throw new Error(
      `missing Mermaid bundle: ${MERMAID_BUNDLE} (run ./.tools/mdbook/bin/mdbook-mermaid install docs)`
    );
  }
  // Mermaid's browser bundle can route through a DOMPurify code path where the
  // imported default is a function-like object in Node. Add minimal no-op
  // compatibility methods so parser-only linting works in headless CI.
  if (typeof Function.prototype.addHook !== "function") {
    Function.prototype.addHook = function addHookShim() {
      return undefined;
    };
  }
  if (typeof Function.prototype.sanitize !== "function") {
    Function.prototype.sanitize = function sanitizeShim(input) {
      return input;
    };
  }

  const code = fs.readFileSync(MERMAID_BUNDLE, "utf8");
  vm.runInThisContext(code, { filename: MERMAID_BUNDLE });
  const mermaid = globalThis.mermaid;
  if (!mermaid || typeof mermaid.parse !== "function") {
    throw new Error("failed to load Mermaid parser from docs/mermaid.min.js");
  }
  mermaid.initialize({ startOnLoad: false, securityLevel: "strict" });
  return mermaid;
}

async function main() {
  if (!fs.existsSync(DOCS_ROOT)) {
    console.error(`docs source directory not found: ${DOCS_ROOT}`);
    process.exit(1);
  }

  const files = walkMarkdownFiles(DOCS_ROOT);
  if (files.length === 0) {
    console.error(`no markdown files found under ${DOCS_ROOT}`);
    process.exit(1);
  }

  const mermaid = loadMermaid();

  const failures = [];
  for (const filePath of files) {
    const source = fs.readFileSync(filePath, "utf8");
    const blocks = extractMermaidBlocks(source, filePath);
    for (const block of blocks) {
      if (block.content === null) {
        failures.push(
          `${block.filePath}:${block.startLine}: mermaid block is not closed with \`\`\``
        );
        continue;
      }
      try {
        await mermaid.parse(block.content);
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        failures.push(`${block.filePath}:${block.startLine}: invalid mermaid: ${message}`);
      }
    }
  }

  if (failures.length > 0) {
    for (const failure of failures) {
      console.error(failure);
    }
    process.exit(1);
  }
}

main().catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`docs mermaid lint failed: ${message}`);
  process.exit(1);
});
