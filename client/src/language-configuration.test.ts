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
      "Sub MySub(x)",
      "Function MyFunc(x)",
      "For i = 1 To 10",
      "Do",
      "Do While x < 10",
      "While x < 10",
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
    ])("matches block-opening line: %s", (line) => {
      expect(increase.test(line)).toBe(true);
    });

    test.each(["If x > 5 Then y = 1", "x = 5", "CallTable(Status)", "NextScan"])(
      "does not match non-block-opening line: %s",
      (line) => {
        expect(increase.test(line)).toBe(false);
      }
    );
  });

  describe("decreaseIndentPattern", () => {
    test.each([
      "EndProg",
      "EndTable",
      "EndSub",
      "EndFunction",
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
    ])("matches block-closing line: %s", (line) => {
      expect(decrease.test(line)).toBe(true);
    });

    test.each(["NextScan", "x = 5", "CallTable(Status)"])(
      "does not match non-block-closing line: %s",
      (line) => {
        expect(decrease.test(line)).toBe(false);
      }
    );
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
      "Sub MySub(x)",
      "Function MyFunc(x)",
      "If x > 5 Then",
      "For i = 1 To 10",
      "Do",
      "While x < 10",
      "Select Case x",
      "#If LoggerType = GRANITE6",
      "#IfDef FINAL Then",
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
      "DoWork(x)",
      "ForecastValue = 1",
      "IfCondition = True",
      "SubTotal = 1",
    ])("does not match non-block-opening line: %s", (line) => {
      expect(start.test(line)).toBe(false);
    });
  });

  describe("end", () => {
    test.each([
      "EndProg",
      "EndTable",
      "EndSub",
      "EndFunction",
      "EndIf",
      "Next",
      "Next i",
      "Loop",
      "Wend",
      "EndSelect",
      "#EndIf",
    ])("matches block-closing line: %s", (line) => {
      expect(end.test(line)).toBe(true);
    });

    test.each([
      "ExitFor",
      "ExitDo",
      "ExitFunction",
      "Exit Sub",
      "Return(x)",
      "NextScan",
      "Loopback = 1",
    ])("does not match non-block-closing line: %s", (line) => {
      expect(end.test(line)).toBe(false);
    });
  });
});
