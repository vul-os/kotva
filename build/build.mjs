// DMTAP spec build: markdown -> single HTML -> PDF via the installed Chrome.
// Syntax highlighting (highlight.js, build-time) + mermaid diagrams (in-page).
// No LaTeX. Usage: node build.mjs   (from the build/ dir)

import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import MarkdownIt from "markdown-it";
import anchor from "markdown-it-anchor";
import hljs from "highlight.js";
import puppeteer from "puppeteer-core";

const __dirname = dirname(fileURLToPath(import.meta.url));
const specDir = join(__dirname, "..");
const meta = JSON.parse(readFileSync(join(__dirname, "meta.json"), "utf8"));
const css = readFileSync(join(__dirname, "style.css"), "utf8");
const hljsCss = readFileSync(join(__dirname, "node_modules/highlight.js/styles/github.css"), "utf8");
const mermaidJs = readFileSync(join(__dirname, "node_modules/mermaid/dist/mermaid.min.js"), "utf8");

const slugify = (s) =>
  s.trim().toLowerCase()
    .replace(/[^\w\s.-]/g, "")
    .replace(/[\s.]+/g, "-")
    .replace(/-+/g, "-").replace(/^-|-$/g, "");

const md = new MarkdownIt({
  html: true,
  linkify: true,
  typographer: true,
  highlight(str, lang) {
    if (lang === "mermaid") {
      // Hand off to mermaid in the browser; escape so the source survives to the DOM.
      const esc = str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
      return `<pre class="mermaid">${esc}</pre>`;
    }
    if (lang && hljs.getLanguage(lang)) {
      try {
        return `<pre><code class="hljs language-${lang}">${hljs.highlight(str, { language: lang }).value}</code></pre>`;
      } catch { /* fall through */ }
    }
    // Untagged / unknown-language blocks (ASCII diagrams, CDDL, preimages): leave plain,
    // never auto-highlight — auto-detection mangles ASCII art and pseudo-grammar.
    const esc = md.utils.escapeHtml(str);
    return `<pre><code class="hljs">${esc}</code></pre>`;
  },
});
md.use(anchor, { slugify, tabIndex: false });

// ---- gather sections in numeric order ----
const files = readdirSync(specDir)
  .filter((f) => /^\d\d-.*\.md$/.test(f))
  .sort();

const toc = [];
let body = "";
for (const f of files) {
  const src = readFileSync(join(specDir, f), "utf8");
  // collect h1/h2 for the TOC (with the same slugs the anchor plugin emits)
  for (const line of src.split("\n")) {
    const m = /^(#{1,2})\s+(.*)$/.exec(line);
    if (m) toc.push({ level: m[1].length, text: m[2].trim(), slug: slugify(m[2].trim()) });
  }
  body += `<section class="section">\n${md.render(src)}\n</section>\n`;
}

const tocHtml = `<nav class="toc"><h2 class="toc-title">Table of Contents</h2><ol>` +
  toc.map((t) => {
    const cls = t.level === 1 ? "toc-h1" : "toc-h2";
    const parts = t.text.match(/^(\d+(?:\.\d+)*\.?)\s+(.*)$/);
    const num = parts ? parts[1] : "";
    const label = parts ? parts[2] : t.text;
    // num / label / dotted-leader inside the <a>; paged.js appends the page number via ::after
    return `<li class="${cls}"><a href="#${t.slug}">` +
      `<span class="num">${md.utils.escapeHtml(num)}</span>` +
      `<span class="label">${md.utils.escapeHtml(label)}</span>` +
      `<span class="lead"></span></a></li>`;
  }).join("") +
  `</ol></nav>`;

const coverHtml = `
<div class="cover">
  <div class="wg-block">
    ${meta.workgroup}<span class="r">${meta.org}</span><br>
    Internet-Draft<span class="r">${meta.date}</span><br>
    Intended status: ${meta.intendedStatus}
  </div>
  <div class="title-wrap">
    <h1 class="doctitle">${meta.title.replace(/\((DMTAP)\)/, "<br>($1)")}</h1>
    <div class="draftid">${meta.draftId}</div>
    <div class="divider"></div>
  </div>
  <div class="abstract-block">
    <h2 class="sub">Abstract</h2>
    <div class="small">${md.renderInline(meta.abstract)}</div>
    <h2 class="sub">Status of This Memo</h2>
    <div class="small">${md.renderInline(meta.status)}</div>
    <h2 class="sub">Requirements Language</h2>
    <div class="small">${md.renderInline(meta.requirements)}</div>
  </div>
</div>`;

const html = `<!doctype html><html lang="en"><head><meta charset="utf-8">
<title>${meta.title}</title>
<style>${hljsCss}</style>
<style>${css}</style>
</head><body>
${coverHtml}
${tocHtml}
${body}
<div class="colophon">Drafted with the help of an LLM by Imran Yusuf Paruk · Durban, South Africa</div>
<script>${mermaidJs}</script>
<script>
  window.__mermaidDone = false;
  (async () => {
    try {
      if (window.mermaid) {
        mermaid.initialize({ startOnLoad: false, theme: "neutral",
          themeVariables: { fontFamily: "Helvetica Neue, Arial, sans-serif", fontSize: "13px" },
          flowchart: { htmlLabels: true, curve: "basis" } });
        await mermaid.run({ querySelector: "pre.mermaid" });
      }
    } catch (e) { console.error("mermaid:", e); }
    window.__mermaidDone = true;
  })();
</script>
</body></html>`;

const htmlPath = join(__dirname, "dmtap.html");
writeFileSync(htmlPath, html);
console.log(`wrote ${htmlPath} (${(html.length / 1024).toFixed(0)} KiB, ${files.length} sections, ${toc.length} toc entries)`);

// ---- render to PDF via the installed Chrome ----
const chrome = process.env.CHROME_PATH ||
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

const browser = await puppeteer.launch({
  executablePath: chrome,
  headless: "new",
  args: ["--no-sandbox", "--font-render-hinting=none"],
});
const page = await browser.newPage();
await page.goto("file://" + htmlPath, { waitUntil: "networkidle0", timeout: 240000 });
await page.waitForFunction("window.__mermaidDone === true", { timeout: 240000 });
await new Promise((r) => setTimeout(r, 400)); // settle fonts/svg layout

// classic Internet-Draft running head + foot (draft-id left, page number right)
const foot = `<div style="width:100%;font-family:Helvetica Neue,Arial,sans-serif;font-size:8pt;color:#6b6b6b;padding:0 20mm;display:flex;justify-content:space-between;">
  <span>${meta.draftId}</span><span></span><span>Page <span class="pageNumber"></span></span></div>`;
const head = `<div style="width:100%;font-family:Helvetica Neue,Arial,sans-serif;font-size:8pt;color:#6b6b6b;padding:0 20mm;display:flex;justify-content:space-between;">
  <span>Internet-Draft</span><span>DMTAP</span><span>${meta.date}</span></div>`;

const outPath = join(specDir, "dmtap.pdf");
await page.pdf({
  path: outPath,
  format: "A4",
  printBackground: true,
  displayHeaderFooter: true,
  headerTemplate: head,
  footerTemplate: foot,
  margin: { top: "22mm", bottom: "20mm", left: "20mm", right: "20mm" },
  timeout: 240000,
});
await browser.close();
console.log(`wrote ${outPath}`);
