/**
 * Tests for the CRBasic operator syntax highlighting patterns in
 * crbasic.tmLanguage.json's "operators" repository entry.
 *
 * TextMate grammars use Oniguruma regex, which supports inline `(?i)`
 * case-insensitivity groups that JS's RegExp engine doesn't parse directly --
 * this loader strips a leading `(?i)` and applies it as the "i" flag
 * instead, so the same case-insensitive matching semantics can be exercised
 * here against Node's regex engine.
 */

import { describe, test, expect } from "vitest";
import * as fs from "fs";
import * as path from "path";

interface TmPattern {
  name: string;
  match: string;
}

interface TmGrammar {
  repository: {
    operators: { patterns: TmPattern[] };
  };
}

function loadGrammar(): TmGrammar {
  const grammarPath = path.join(__dirname, "..", "syntaxes", "crbasic.tmLanguage.json");
  const raw = fs.readFileSync(grammarPath, "utf-8");
  return JSON.parse(raw) as TmGrammar;
}

function toRegExp(match: string): RegExp {
  if (match.startsWith("(?i)")) {
    return new RegExp(match.slice("(?i)".length), "i");
  }
  return new RegExp(match);
}

function operatorPatterns(): TmPattern[] {
  return loadGrammar().repository.operators.patterns;
}

/**
 * True only if some pattern matches `text` as one complete token, not just
 * a substring -- e.g. `+=` must be recognized as a single compound-assignment
 * operator, not merely contain a `+` that happens to match the plain
 * addition pattern.
 */
function isHighlightedAsOperator(text: string): boolean {
  return operatorPatterns().some((pattern) => {
    const match = toRegExp(pattern.match).exec(text);
    return match !== null && match.index === 0 && match[0].length === text.length;
  });
}

describe("crbasic.tmLanguage.json operator highlighting", () => {
  test.each(["=", "<>", "<", ">", "<=", ">=", "+", "-", "*", "/", "^"])(
    "highlights the already-covered %s operator as one complete token",
    (text) => {
      expect(isHighlightedAsOperator(text)).toBe(true);
    }
  );

  test.each(["AND", "and", "OR", "or", "NOT", "not", "XOR", "xor"])(
    "highlights the already-covered %s operator case-insensitively",
    (text) => {
      expect(isHighlightedAsOperator(text)).toBe(true);
    }
  );

  test.each([
    ["&", "string concatenation"],
    ["\\", "integer division"],
    ["<<", "bit-shift left"],
    [">>", "bit-shift right"],
    ["@", "address-of pointer"],
    ["!", "dereference pointer"],
    ["Mod", "remainder"],
    ["mod", "remainder (lowercase)"],
    ["Imp", "logical implication"],
    ["imp", "logical implication (lowercase)"],
    ["+=", "compound add-assign"],
    ["-=", "compound subtract-assign"],
    ["*=", "compound multiply-assign"],
    ["/=", "compound divide-assign"],
    ["^=", "compound power-assign"],
    ["&=", "compound concatenate-assign"],
    ["\\=", "compound integer-divide-assign"],
  ])("highlights the %s operator (%s)", (text) => {
    expect(isHighlightedAsOperator(text)).toBe(true);
  });
});

interface TmGrammarWithNumbers extends TmGrammar {
  repository: TmGrammar["repository"] & {
    numbers: { patterns: TmPattern[] };
  };
}

function numberPatterns(): TmPattern[] {
  return (loadGrammar() as TmGrammarWithNumbers).repository.numbers.patterns;
}

function isHighlightedAsNumber(text: string): boolean {
  return numberPatterns().some((pattern) => {
    const match = toRegExp(pattern.match).exec(text);
    return match !== null && match.index === 0 && match[0].length === text.length;
  });
}

describe("crbasic.tmLanguage.json numeric literal highlighting", () => {
  test.each(["123", "3.14", "1.0e-5"])(
    "highlights the already-covered %s numeric literal as one complete token",
    (text) => {
      expect(isHighlightedAsNumber(text)).toBe(true);
    }
  );

  test.each([
    ["&HFF", "hexadecimal literal"],
    ["&hff", "lowercase hexadecimal literal"],
    ["&B1010", "binary literal"],
    ["&b1010", "lowercase binary literal"],
  ])("highlights the %s (%s)", (text) => {
    expect(isHighlightedAsNumber(text)).toBe(true);
  });

  test("does not swallow the concatenation operator when no digit follows (A&Bvar)", () => {
    // `&B` without a following binary digit must stay the concatenation
    // operator, matching the lexer's own fallback behavior.
    expect(isHighlightedAsNumber("&Bvar")).toBe(false);
  });
});

interface TmStringRule {
  name: string;
  begin: string;
  end: string;
  patterns?: TmPattern[];
}

interface TmGrammarWithStrings extends TmGrammar {
  repository: TmGrammar["repository"] & {
    strings: TmStringRule;
  };
}

function stringRule(): TmStringRule {
  return (loadGrammar() as TmGrammarWithStrings).repository.strings;
}

describe("crbasic.tmLanguage.json string literal highlighting", () => {
  // CRBasic string literals have no backslash-escape mechanism at all --
  // `scan_string` (crates/crbasic-parser/src/lexer/scanner.rs) treats `\`
  // as a plain, literal character, so a Windows path like `"C:\network"`
  // must render as one uncolored string, not partially colored as if `\n`
  // were an escape sequence.
  test("does not highlight backslash sequences as escape characters", () => {
    expect(stringRule().patterns ?? []).toEqual([]);
  });

  // `scan_string` stops at the end of the line for an unterminated string
  // (a forgotten closing quote must not swallow the rest of the file) --
  // the grammar's `end` pattern needs the same line-boundary fallback, or
  // an unterminated string would visually swallow every following line up
  // to the next stray `"` instead of stopping where the lexer does.
  test("does not span an unterminated string past the end of its line", () => {
    expect(stringRule().end).toMatch(/\$/);
  });
});
