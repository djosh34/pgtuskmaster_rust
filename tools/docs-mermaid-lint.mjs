#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const REPO_ROOT = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const DOCS_ROOT = path.join(REPO_ROOT, "docs", "src");

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

function escapedNewlineLine(content, startLine) {
  const lines = content.split("\n");
  for (let idx = 0; idx < lines.length; idx += 1) {
    if (lines[idx].includes("\\n")) {
      return startLine + idx;
    }
  }
  return null;
}

async function loadMermaid() {
  let mermaidModule;
  try {
    mermaidModule = await import("mermaid");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(
      `failed to load Mermaid npm package (${message}); run ./tools/install-docs-node-deps.sh`
    );
  }

  const mermaid = mermaidModule.default;
  if (!mermaid || typeof mermaid.parse !== "function") {
    throw new Error("failed to load Mermaid parser from npm package");
  }
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
  // This lint only checks Mermaid syntax. "strict" routes through DOMPurify
  // in Node and introduces headless-only failures unrelated to diagram validity.
  mermaid.initialize({ startOnLoad: false, securityLevel: "loose" });
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

  const mermaid = await loadMermaid();

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
      const escapedLine = escapedNewlineLine(block.content, block.startLine);
      if (escapedLine !== null) {
        failures.push(
          `${block.filePath}:${escapedLine}: mermaid contains literal \\n; use <br/> for line breaks`
        );
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
