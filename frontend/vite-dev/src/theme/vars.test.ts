import { describe, expect, it } from "vitest";
import { generateThemeVarsCss, isValidVarName, sanitizeVarValue, toHex } from "./vars";

describe("toHex", () => {
  it("normalises opaque hex and rgb colours", () => {
    expect(toHex("#ABC")).toBe("#aabbcc");
    expect(toHex(" #7a4a24 ")).toBe("#7a4a24");
    expect(toHex("rgb(255, 0, 16)")).toBe("#ff0010");
    expect(toHex("rgba(1, 2, 3, 1)")).toBe("#010203");
  });

  it("refuses values a colour input can't represent", () => {
    expect(toHex("rgba(0, 0, 0, 0.5)")).toBeNull();
    expect(toHex("linear-gradient(red, blue)")).toBeNull();
    expect(toHex("tomato")).toBeNull();
  });
});

describe("override sanitising", () => {
  it("strips declaration-breaking characters", () => {
    expect(sanitizeVarValue("red; } body { display: none")).toBe("red  body  display: none");
  });

  it("accepts only kebab-case custom property names", () => {
    expect(isValidVarName("--accent")).toBe(true);
    expect(isValidVarName("--board-light")).toBe(true);
    expect(isValidVarName("color")).toBe(false);
    expect(isValidVarName("--x}{")).toBe(false);
  });

  it("compiles valid, non-empty overrides into a :root block", () => {
    const css = generateThemeVarsCss({ "--accent": "#ff0000", bad: "red", "--text": "  " });
    expect(css).toBe(":root {\n  --accent: #ff0000;\n}");
    expect(generateThemeVarsCss({})).toBe("");
  });
});
