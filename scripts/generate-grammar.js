/**
 * Generates crates/crbasic-parser/src/keywords_generated.rs and
 * client/syntaxes/crbasic.tmLanguage.json from the single source of truth
 * crates/crbasic-parser/keywords.json.
 *
 * Run `node scripts/generate-grammar.js` to regenerate both files, or
 * `node scripts/generate-grammar.js --check` to verify the checked-in
 * files are already up to date (used in CI, exits 1 with no write on
 * mismatch).
 */

const fs = require("fs");
const path = require("path");

const repoRoot = path.join(__dirname, "..");
const keywordsPath = path.join(
  repoRoot,
  "crates",
  "crbasic-parser",
  "keywords.json"
);
const rustOutputPath = path.join(
  repoRoot,
  "crates",
  "crbasic-parser",
  "src",
  "keywords_generated.rs"
);
const grammarOutputPath = path.join(
  repoRoot,
  "client",
  "syntaxes",
  "crbasic.tmLanguage.json"
);

function byCategory(entries) {
  const grouped = new Map();
  for (const entry of entries) {
    if (!grouped.has(entry.category)) {
      grouped.set(entry.category, []);
    }
    grouped.get(entry.category).push(entry.name);
  }
  return grouped;
}

function alternation(names) {
  // `\b` needs a word-char/non-word-char transition on both sides, but `#`
  // is itself a non-word char -- a leading `\b` right before e.g. `#If`
  // would never match (whitespace/line-start before `#` is non-word on
  // both sides). Preprocessor directive names are the only entries that
  // start with `#`, so drop the leading boundary just for them; the
  // trailing `\b` still works since directive names end in a word char.
  const leadingBoundary = names[0].startsWith("#") ? "" : "\\b";
  return `(?i)${leadingBoundary}(${names.join("|")})\\b`;
}

function generateRustSource(keywords) {
  const toConstArray = (constName, entries) => {
    const items = entries
      .map((e) => `    ("${e.name}", "${e.category}"),`)
      .join("\n");
    return `pub const ${constName}: &[(&str, &str)] = &[\n${items}\n];\n`;
  };

  return (
    "// GENERATED FILE - do not edit by hand.\n" +
    "// Source: crates/crbasic-parser/keywords.json\n" +
    "// Regenerate with: node scripts/generate-grammar.js\n\n" +
    toConstArray("LANGUAGE_KEYWORDS", keywords.languageKeywords) +
    "\n" +
    toConstArray("BUILTIN_FUNCTIONS", keywords.builtinFunctions)
  );
}

function generateGrammar(keywords) {
  const kw = byCategory(keywords.languageKeywords);
  const fn = byCategory(keywords.builtinFunctions);

  const scanNames = [
    ...(fn.get("scan") ?? []),
    ...(kw.get("scan") ?? []),
  ];

  const grammar = {
    $schema:
      "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
    name: "CRBasic",
    scopeName: "source.crbasic",
    fileTypes: [
      ".cr1",
      ".cr1x",
      ".cr2",
      ".cr3",
      ".cr5",
      ".cr6",
      ".cr8",
      ".cr9",
      ".cr9x",
      ".c9x",
      ".cr300",
      ".crb",
      ".dld",
    ],
    patterns: [
      { include: "#comments" },
      { include: "#line-continuation" },
      { include: "#strings" },
      { include: "#numbers" },
      { include: "#preprocessor-keywords" },
      { include: "#control-keywords" },
      { include: "#declaration-keywords" },
      { include: "#structure-keywords" },
      { include: "#builtin-functions-measurement" },
      { include: "#builtin-functions-communication" },
      { include: "#builtin-functions-data" },
      { include: "#builtin-functions-general" },
      { include: "#operators" },
      { include: "#identifiers" },
    ],
    repository: {
      comments: {
        name: "comment.line.single-quote.crbasic",
        match: "'.*$",
      },
      "line-continuation": {
        name: "punctuation.separator.continuation.crbasic",
        match: "\\s_$",
      },
      strings: {
        name: "string.quoted.double.crbasic",
        begin: '"',
        end: '"',
        patterns: [
          {
            name: "constant.character.escape.crbasic",
            match: "\\\\.",
          },
        ],
      },
      numbers: {
        patterns: [
          {
            name: "constant.numeric.float.crbasic",
            match: "\\b\\d+\\.\\d+([eE][+-]?\\d+)?\\b",
          },
          {
            name: "constant.numeric.integer.crbasic",
            match: "\\b\\d+\\b",
          },
        ],
      },
      "preprocessor-keywords": {
        name: "keyword.control.preprocessor.crbasic",
        match: alternation(kw.get("preprocessor")),
      },
      "control-keywords": {
        name: "keyword.control.crbasic",
        match: alternation(kw.get("control")),
      },
      "declaration-keywords": {
        name: "storage.type.crbasic",
        match: alternation(kw.get("declaration")),
      },
      "structure-keywords": {
        patterns: [
          {
            name: "keyword.control.program.crbasic",
            match: alternation(kw.get("program")),
          },
          {
            name: "keyword.control.datatable.crbasic",
            match: alternation(kw.get("datatable")),
          },
          {
            name: "keyword.control.function.crbasic",
            match: alternation(kw.get("function")),
          },
          {
            name: "keyword.control.scan.crbasic",
            match: alternation(scanNames),
          },
        ],
      },
      "builtin-functions-measurement": {
        name: "support.function.measurement.crbasic",
        match: alternation(fn.get("measurement")),
      },
      "builtin-functions-communication": {
        name: "support.function.communication.crbasic",
        match: alternation(fn.get("communication")),
      },
      "builtin-functions-data": {
        name: "support.function.data.crbasic",
        match: alternation(fn.get("data")),
      },
      "builtin-functions-general": {
        patterns: [
          {
            name: "support.function.string.crbasic",
            match: alternation(fn.get("string")),
          },
          {
            name: "support.function.math.crbasic",
            match: alternation(fn.get("math")),
          },
          {
            name: "support.function.time.crbasic",
            match: alternation(fn.get("time")),
          },
          {
            name: "support.function.logical.crbasic",
            match: alternation(fn.get("logical")),
          },
        ],
      },
      operators: {
        patterns: [
          {
            name: "keyword.operator.comparison.crbasic",
            match: "(=|<>|<|>|<=|>=)",
          },
          {
            name: "keyword.operator.arithmetic.crbasic",
            match: "(\\+|-|\\*|/|\\^|MOD)",
          },
          {
            name: "keyword.operator.logical.crbasic",
            match: "(?i)\\b(AND|OR|NOT|XOR)\\b",
          },
          {
            name: "constant.language.boolean.crbasic",
            match: "(?i)\\b(True|False)\\b",
          },
          {
            name: "keyword.operator.assignment.crbasic",
            match: "=",
          },
        ],
      },
      identifiers: {
        name: "variable.other.crbasic",
        match: "\\b[A-Za-z_][A-Za-z0-9_]*\\b",
      },
    },
  };

  return JSON.stringify(grammar, null, 2) + "\n";
}

function main() {
  const checkOnly = process.argv.includes("--check");
  const keywords = JSON.parse(fs.readFileSync(keywordsPath, "utf8"));

  const outputs = [
    { path: rustOutputPath, content: generateRustSource(keywords) },
    { path: grammarOutputPath, content: generateGrammar(keywords) },
  ];

  if (checkOnly) {
    let stale = false;
    for (const { path: outPath, content } of outputs) {
      const current = fs.existsSync(outPath)
        ? fs.readFileSync(outPath, "utf8")
        : null;
      if (current !== content) {
        console.error(`Stale generated file: ${path.relative(repoRoot, outPath)}`);
        stale = true;
      }
    }
    if (stale) {
      console.error(
        "Run `node scripts/generate-grammar.js` to regenerate, then commit the result."
      );
      process.exit(1);
    }
    console.log("Generated files are up to date.");
    return;
  }

  for (const { path: outPath, content } of outputs) {
    fs.writeFileSync(outPath, content);
    console.log(`Wrote ${path.relative(repoRoot, outPath)}`);
  }
}

main();
