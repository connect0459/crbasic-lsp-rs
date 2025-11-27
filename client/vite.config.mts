import { defineConfig } from "vite";
import { resolve } from "path";
import { builtinModules } from "module";

export default defineConfig({
  build: {
    lib: {
      entry: resolve(__dirname, "src/extension.ts"),
      name: "CRBasicLSP",
      fileName: "extension",
      formats: ["cjs"],
    },
    rollupOptions: {
      external: [
        "vscode",
        "vscode-languageclient/node",
        // Node.js built-in modules
        ...builtinModules,
        ...builtinModules.map((m) => `node:${m}`),
      ],
      output: {
        globals: {
          vscode: "vscode",
        },
      },
    },
    outDir: "dist",
    sourcemap: true,
    minify: false, // Keep readable for debugging
    target: "node18", // Target Node.js 18+
  },
  test: {
    globals: true,
    environment: "node",
    coverage: {
      provider: "v8",
      reporter: ["text", "html", "lcov"],
      include: ["src/**/*.ts"],
      exclude: ["src/**/*.test.ts", "src/**/*.spec.ts"],
      lines: 80,
      branches: 75,
      functions: 90,
      statements: 80,
    },
  },
});
