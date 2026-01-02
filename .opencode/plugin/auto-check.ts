import type { Plugin } from "@opencode-ai/plugin";

export const AutoCheckPlugin: Plugin = async ({ $ }) => {
  return {
    // 對應 JSON 中的 "PostToolUse"
    "tool.execute.after": async (input, output) => {
      // 1. 定義觸發條件 (對應 JSON 中的 "matcher": "Write|Edit|Replace")
      // OpenCode 的工具名稱通常是小寫，這裡列出常見的文件修改工具名稱
      const modificationTools = ["write", "edit", "replace", "apply_diff"];

      // 檢查當前使用的工具是否包含上述關鍵字
      const isModification = modificationTools.some((t) =>
        input.tool.includes(t)
      );

      if (isModification) {
        console.log(
          `[AutoCheck] Detect file modification by tool: ${input.tool}`
        );

        try {
          // 2. 執行檢查指令 (對應 JSON 中的 "command")
          // OpenCode 的 $ 會使用系統 Shell 執行，直接貼上原本的指令即可
          await $`if [ -f Cargo.toml ]; then cargo check; fi && if [ -f package.json ]; then npm run check; fi`;

          console.log("[AutoCheck] All checks passed successfully.");
        } catch (e) {
          // 如果檢查失敗，會在 Console 報錯，AI 下一步可能會看到錯誤
          console.error("[AutoCheck] Check failed:", e);

          // 進階技巧：如果您希望這會導致工具報錯並讓 AI 立即修正，可以 throw error
          // throw new Error(`Auto-check failed after edit: ${e.message}`);
        }
      }
    },
  };
};
