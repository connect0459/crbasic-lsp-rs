/**
 * Tests for the CRBasic auto-indent rules in language-configuration.json.
 *
 * VSCode applies these regexes directly against a single line of text to
 * decide whether to indent the next line or dedent the line just typed.
 * These tests exercise that same regex/line contract so a bad pattern is
 * caught here instead of by manually pressing Enter in the editor.
 */

import { describe, test, expect } from "vitest";
import * as fs from "fs";
import * as path from "path";

interface IndentPattern {
  pattern: string;
  flags: string;
}

interface FoldingMarkers {
  start: string;
  end: string;
}

interface LanguageConfiguration {
  brackets: [string, string][];
  autoClosingPairs: { open: string; close: string }[];
  surroundingPairs: [string, string][];
  indentationRules: {
    increaseIndentPattern: IndentPattern;
    decreaseIndentPattern: IndentPattern;
  };
  folding: {
    markers: FoldingMarkers;
  };
}

function loadConfig(): LanguageConfiguration {
  const configPath = path.join(__dirname, "..", "language-configuration.json");
  const raw = fs.readFileSync(configPath, "utf-8");
  return JSON.parse(raw) as LanguageConfiguration;
}

function toRegExp(p: IndentPattern): RegExp {
  return new RegExp(p.pattern, p.flags);
}

describe("language-configuration.json indentationRules", () => {
  const config = loadConfig();
  const increase = toRegExp(config.indentationRules.increaseIndentPattern);
  const decrease = toRegExp(config.indentationRules.decreaseIndentPattern);

  describe("increaseIndentPattern", () => {
    test.each([
      "BeginProg",
      "DataTable(Test, 1, -1)",
      "ConstTable(NewConstTable, 0)",
      "ApplyAndRestartSequence",
      "ShutDownBegin",
      "StructureType TempRHSensor",
      "Sub MySub(x)",
      "Function MyFunc(x)",
      "For i = 1 To 10",
      "Do",
      "Do While x < 10",
      "While x < 10",
      "Scan(1, Sec, 0, 0)",
      "SubScan(0.1, Sec, 5)",
      "SlowSequence",
      "Select Case x",
      "Else",
      "ElseIf x > 5 Then",
      "Case 1",
      "If x > 5 Then",
      "  If x > 5 Then",
      "#If LoggerType = GRANITE6",
      "#If Add107 Then",
      "#IfDef FINAL Then",
      "#ElseIf LoggerType = CR1000",
      "#Else",
      'DisplayMenu("Menu1", 1, 1)',
      'SubMenu("Sub1")',
      'WebPageBegin("Page1", 1)',
      "ModemHangup(ComC1)",
      "VoiceBeg",
    ])("matches block-opening line: %s", (line) => {
      expect(increase.test(line)).toBe(true);
    });

    test.each([
      "If x > 5 Then y = 1",
      "x = 5",
      "CallTable Status",
      "NextScan",
      "NextSubScan",
      "ContinueScan",
      "ScanValue = 5",
      "EndMenu",
      "EndSubMenu",
      'MenuItem("Item1")',
      "WebPageEnd",
      "EndModemHangup",
      "EndVoice",
    ])("does not match non-block-opening line: %s", (line) => {
      expect(increase.test(line)).toBe(false);
    });
  });

  describe("decreaseIndentPattern", () => {
    test.each([
      "EndProg",
      "EndTable",
      "EndConstTable",
      "EndApplyAndRestartSequence",
      "ShutDownEnd",
      "EndStructureType",
      "EndSub",
      "EndFunction",
      "NextScan",
      "NextSubScan",
      "EndSequence",
      "EndSelect",
      "EndIf",
      "Next",
      "Next i",
      "Loop",
      "Wend",
      "Else",
      "ElseIf x > 5 Then",
      "Case 1",
      "Case Else",
      "#EndIf",
      "#ElseIf LoggerType = CR1000",
      "#Else",
      "EndMenu",
      "EndSubMenu",
      "WebPageEnd",
      "EndModemHangup",
      "EndVoice",
    ])("matches block-closing line: %s", (line) => {
      expect(decrease.test(line)).toBe(true);
    });

    test.each([
      "x = 5",
      "CallTable Status",
      "ContinueScan",
      "Scan(1, Sec, 0, 0)",
      "SubScan(0.1, Sec, 5)",
      'DisplayMenu("Menu1", 1, 1)',
      'SubMenu("Sub1")',
      'WebPageBegin("Page1", 1)',
      "ModemHangup(ComC1)",
      "VoiceBeg",
    ])("does not match non-block-closing line: %s", (line) => {
      expect(decrease.test(line)).toBe(false);
    });
  });
});

describe("language-configuration.json folding.markers", () => {
  const config = loadConfig();
  const start = new RegExp(config.folding.markers.start);
  const end = new RegExp(config.folding.markers.end);

  describe("start", () => {
    test.each([
      "BeginProg",
      "DataTable(Test, 1, -1)",
      "ConstTable(NewConstTable, 0)",
      "ApplyAndRestartSequence",
      "ShutDownBegin",
      "StructureType TempRHSensor",
      "Sub MySub(x)",
      "Function MyFunc(x)",
      "If x > 5 Then",
      "For i = 1 To 10",
      "Do",
      "While x < 10",
      "Scan(1, Sec, 0, 0)",
      "SubScan(0.1, Sec, 5)",
      "SlowSequence",
      "Select Case x",
      "#If LoggerType = GRANITE6",
      "#IfDef FINAL Then",
      'DisplayMenu("Menu1", 1, 1)',
      'SubMenu("Sub1")',
      'WebPageBegin("Page1", 1)',
      "ModemHangup(ComC1)",
      "VoiceBeg",
    ])("matches block-opening line: %s", (line) => {
      expect(start.test(line)).toBe(true);
    });

    test.each([
      "ExitFor",
      "ExitDo",
      "ExitFunction",
      "Exit Sub",
      "Return(x)",
      "NextScan",
      "NextSubScan",
      "ContinueScan",
      "DoWork(x)",
      "ForecastValue = 1",
      "IfCondition = True",
      "SubTotal = 1",
      "ScanValue = 5",
      "EndMenu",
      "EndSubMenu",
      'MenuItem("Item1")',
      "WebPageEnd",
      "EndModemHangup",
      "EndVoice",
    ])("does not match non-block-opening line: %s", (line) => {
      expect(start.test(line)).toBe(false);
    });
  });

  describe("end", () => {
    test.each([
      "EndProg",
      "EndTable",
      "EndConstTable",
      "EndApplyAndRestartSequence",
      "ShutDownEnd",
      "EndStructureType",
      "EndSub",
      "EndFunction",
      "NextScan",
      "NextSubScan",
      "EndSequence",
      "EndIf",
      "Next",
      "Next i",
      "Loop",
      "Wend",
      "EndSelect",
      "#EndIf",
      "EndMenu",
      "EndSubMenu",
      "WebPageEnd",
      "EndModemHangup",
      "EndVoice",
    ])("matches block-closing line: %s", (line) => {
      expect(end.test(line)).toBe(true);
    });

    test.each([
      "ExitFor",
      "ExitDo",
      "ExitFunction",
      "Exit Sub",
      "Return(x)",
      "ContinueScan",
      "Scan(1, Sec, 0, 0)",
      "SubScan(0.1, Sec, 5)",
      "Loopback = 1",
      'DisplayMenu("Menu1", 1, 1)',
      'SubMenu("Sub1")',
      'WebPageBegin("Page1", 1)',
      "ModemHangup(ComC1)",
      "VoiceBeg",
    ])("does not match non-block-closing line: %s", (line) => {
      expect(end.test(line)).toBe(false);
    });
  });
});

describe("language-configuration.json bracket configuration", () => {
  const config = loadConfig();

  // CRBasic has no bracket-array syntax -- array access and function calls
  // both use `Name(...)`, and `[`/`]` have no meaning in the language at all.
  test("does not pair square brackets", () => {
    expect(config.brackets).not.toContainEqual(["[", "]"]);
  });

  test("does not auto-close square brackets", () => {
    expect(config.autoClosingPairs).not.toContainEqual({ open: "[", close: "]" });
  });

  test("does not surround selections with square brackets", () => {
    expect(config.surroundingPairs).not.toContainEqual(["[", "]"]);
  });

  test("still pairs parentheses", () => {
    expect(config.brackets).toContainEqual(["(", ")"]);
  });

  // `{v1, v2, ...}` is the real CRBasic brace-list array initializer syntax
  // (e.g. `Public MyArray(3) = {3, 6, 9}`), lexed and parsed since it was
  // added to the parser -- the editor config never gained matching pairing.
  test("pairs curly braces (array literal initializer syntax)", () => {
    expect(config.brackets).toContainEqual(["{", "}"]);
  });

  test("auto-closes curly braces", () => {
    expect(config.autoClosingPairs).toContainEqual({ open: "{", close: "}" });
  });

  test("surrounds selections with curly braces", () => {
    expect(config.surroundingPairs).toContainEqual(["{", "}"]);
  });
});
