/**
 * StreamGrab E2E 入口
 *
 * 1. 以 VITE_E2E_MOCK=1 启动 Vite dev server（注入 Tauri bridge mock）
 * 2. 启动 chrome-devtools-mcp 服务器（headless Chrome，--isolated 临时 profile）
 * 3. 顺序执行 e2e/tests/* 中的全部场景，汇总报告，失败退出码 1
 *
 * 运行：npm run test:e2e
 */

import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { Driver, McpClient, ROOT, tests } from "./runner-lib.mjs";
import "./tests/app-shell.mjs";
import "./tests/add-task.mjs";
import "./tests/task-management.mjs";
import "./tests/download-lifecycle.mjs";
import "./tests/settings.mjs";
import "./tests/presets.mjs";
import "./tests/clipboard.mjs";

const E2E_PORT = process.env.E2E_PORT || "5173";
const BASE_URL = `http://localhost:${E2E_PORT}`;

async function waitForServer(timeoutMs = 120000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(BASE_URL);
      if (res.ok || res.status === 404) return;
    } catch {
      /* not ready */
    }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`Vite dev server 未在 ${timeoutMs}ms 内就绪：${BASE_URL}`);
}

async function main() {
  let vite = null;
  let viteExited = null;

  try {
    // 1. Vite dev server（e2e mock 注入）
    vite = spawn(
      process.execPath,
      [
        resolve(ROOT, "node_modules/vite/bin/vite.js"),
        "--port",
        E2E_PORT,
        "--strictPort",
      ],
      {
        env: { ...process.env, VITE_E2E_MOCK: "1" },
        stdio: ["ignore", "ignore", "pipe"],
      },
    );
    vite.on("exit", (code) => {
      viteExited = code;
    });
    vite.stderr.on("data", (d) => process.stderr.write("[vite] " + d));

    await waitForServer();

    // 2. chrome-devtools MCP 服务器
    const client = new McpClient();
    await client.start();
    const driver = new Driver(client);

    // 3. 执行全部测试
    let passed = 0;
    const failures = [];
    const overallStart = Date.now();

    for (const t of tests) {
      const t0 = Date.now();
      try {
        await t.fn(driver);
        passed++;
        console.log(`  ok   ${t.name} (${Date.now() - t0}ms)`);
      } catch (e) {
        failures.push({ name: t.name, error: e });
        console.error(`FAIL   ${t.name} (${Date.now() - t0}ms)`);
        console.error(
          "       " +
            String(e?.stack || e?.message || e)
              .split("\n")
              .join("\n       "),
        );
        await driver.screenshot(
          "failure-" + t.name.replace(/[^\w-]/g, "_").slice(0, 60),
        );
      }
    }

    await client.stop();

    const totalMs = Date.now() - overallStart;
    console.log("");
    console.log(`结果：${passed}/${tests.length} 通过（${totalMs}ms）`);
    if (failures.length > 0) {
      console.log("失败用例：");
      for (const f of failures) {
        console.log(`  - ${f.name}`);
      }
      process.exitCode = 1;
    }
  } catch (e) {
    console.error("E2E 运行失败：", e);
    if (viteExited !== null) {
      console.error(
        `Vite dev server 提前退出（code=${viteExited}）。请确认 ${BASE_URL} 端口未被占用。`,
      );
    }
    process.exitCode = 1;
  } finally {
    if (vite) vite.kill();
  }
}

main();
