#!/usr/bin/env bun

import { $ } from "bun";

type JJConfigValue = {
  name: string;
  value: string;
  source: string;
  path: string | null;
  is_overridden: boolean;
};

type Color = {
  fg?: string;
  bg?: string;
  bold?: boolean;
  dim?: boolean;
  italic?: boolean;
  underlined?: boolean;
  reverse?: boolean;
};

async function parseJJOutputJson<T>(): Promise<T[]> {
  const result =
    await $`jj config list --color never --include-defaults colors -T 'json(self) ++ "\n"'`.text();

  return result
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
}

function reconstructColors(configValues: JJConfigValue[]): Record<string, Color> {
  let colors: Record<string, Color> = {};

  for (let configValue of configValues) {
    let [_, scope, attr, ...rest] = configValue.name.split(".");
    if (!scope || rest.length > 0) throw Error("invalid config");

    scope = scope.replaceAll('"', "");
    attr ??= "fg";

    let color = (colors[scope] ??= {}) as any;
    color[attr] = configValue.value;
  }
  return colors;
}

// ANSI color lookup table (colors 0-15)
const ANSI_COLORS: Record<string, string> = {
  black: "#000000",
  red: "#800000",
  green: "#008000",
  yellow: "#808000",
  blue: "#000080",
  magenta: "#800080",
  cyan: "#008080",
  white: "#c0c0c0",
  "bright black": "#808080",
  "bright red": "#ff0000",
  "bright green": "#00ff00",
  "bright yellow": "#ffff00",
  "bright blue": "#0000ff",
  "bright magenta": "#ff00ff",
  "bright cyan": "#00ffff",
  "bright white": "#ffffff",
  default: "#ffffff",
};

function ansi256ToHex(index: number): string {
  // Colors 0-15: standard ANSI colors
  if (index <= 15) {
    const names = [
      "black",
      "red",
      "green",
      "yellow",
      "blue",
      "magenta",
      "cyan",
      "white",
      "bright black",
      "bright red",
      "bright green",
      "bright yellow",
      "bright blue",
      "bright magenta",
      "bright cyan",
      "bright white",
    ];
    return ANSI_COLORS[names[index]!]!;
  }

  // Colors 16-231: 6x6x6 RGB cube
  if (index >= 16 && index <= 231) {
    const cubeIndex = index - 16;
    const r = Math.floor(cubeIndex / 36);
    const g = Math.floor((cubeIndex % 36) / 6);
    const b = cubeIndex % 6;

    const valueMap = [0, 95, 135, 175, 215, 255];
    const rVal = valueMap[r]!;
    const gVal = valueMap[g]!;
    const bVal = valueMap[b]!;

    return `#${rVal.toString(16).padStart(2, "0")}${gVal.toString(16).padStart(2, "0")}${bVal.toString(16).padStart(2, "0")}`;
  }

  // Colors 232-255: grayscale
  if (index >= 232 && index <= 255) {
    const gray = 8 + 10 * (index - 232);
    return `#${gray.toString(16).padStart(2, "0")}${gray.toString(16).padStart(2, "0")}${gray.toString(16).padStart(2, "0")}`;
  }

  throw new Error(`unknown ansi color index: ${index}`);
}

function jjColorToHex(color: string): string {
  // Hex color passthrough (strip alpha if present)
  if (color.startsWith("#")) {
    return color.slice(0, 7);
  }

  // ANSI 256 color format
  if (color.startsWith("ansi-color-")) {
    const index = parseInt(color.slice(11), 10);
    if (!isNaN(index) && index >= 0 && index <= 255) {
      return ansi256ToHex(index);
    }
  }

  // Named colors (including bright variants)
  const lowerColor = color.toLowerCase();
  if (lowerColor in ANSI_COLORS) {
    return ANSI_COLORS[lowerColor]!;
  }

  throw new Error(`unknown color: ${color}`);
}

function buildZedConfigs(colors: Record<string, Color>): any[] {
  let zedConfigs = [];

  for (let scope in colors) {
    let color = colors[scope];

    let scopeLabels = scope.split(" ");
    if (scopeLabels.length != 1) continue;

    let label = scopeLabels[0];

    let zedConfig: any = {
      token_type: label,
    };
    if (color?.bold) zedConfig["font_weight"] = "bold";
    if (color?.italic) zedConfig["font_style"] = "italic";
    if (color?.underlined) zedConfig["underline"] = true;
    if (color?.fg) zedConfig["foreground_color"] = jjColorToHex(color.fg);
    if (color?.bg) zedConfig["background_color"] = jjColorToHex(color.bg);
    zedConfigs.push(zedConfig);
  }

  return zedConfigs;
}

async function run(): Promise<void> {
  let configValues = await parseJJOutputJson<JJConfigValue>();
  let colors = reconstructColors(configValues);
  let zedConfigs = buildZedConfigs(colors);

  const outDir = `${import.meta.dir}/out`;
  await Bun.write(`${outDir}/colors.json`, JSON.stringify(colors, null, 2));
  console.log(colors);

  const outputPath = `${outDir}/semantic_token_rules.json`;
  await Bun.write(outputPath, JSON.stringify(zedConfigs, null, 2));
  console.log(`Wrote ${zedConfigs.length} token rules to ${outputPath}`);
}

run().catch((error) => {
  console.error("Error executing jj command:");
  console.error(error);
  process.exit(1);
});
