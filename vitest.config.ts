import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  resolve: {
    alias: {
      "@stellar-escrow/sdk": path.resolve(__dirname, "sdk/src/index.ts"),
    },
  },
  test: {
    include: ["sdk/src/**/*.test.ts", "tests/**/*.test.ts"],
    exclude: ["**/node_modules/**", "**/dist/**", "frontend/**"],
    environment: "node",
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      include: ["sdk/src/**"],
      exclude: ["**/*.test.ts", "**/index.ts"],
    },
  },
});
