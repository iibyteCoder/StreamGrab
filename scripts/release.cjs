#!/usr/bin/env node

/**
 * 发布脚本
 *
 * 自动化发布流程：
 * 1. 更新版本号（package.json, Cargo.toml, tauri.conf.json）
 * 2. 创建发行说明文档（如果不存在）
 * 3. 更新 CLAUDE.md 中的功能状态
 *
 * 用法：
 *   node scripts/release.js <version>
 *   node scripts/release.js 0.5.0
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const ROOT_DIR = path.join(__dirname, '..');
const PACKAGE_JSON = path.join(ROOT_DIR, 'package.json');
const CARGO_TOML = path.join(ROOT_DIR, 'src-tauri', 'Cargo.toml');
const TAURI_CONF = path.join(ROOT_DIR, 'src-tauri', 'tauri.conf.json');
const RELEASE_DIR = path.join(ROOT_DIR, 'docs', 'releases');

/**
 * 解析版本号
 */
function parseVersion(version) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/);
  if (!match) {
    throw new Error(`无效的版本号格式: ${version}`);
  }
  return {
    major: parseInt(match[1], 10),
    minor: parseInt(match[2], 10),
    patch: parseInt(match[3], 10),
    full: match[0],
  };
}

/**
 * 更新 package.json
 */
function updatePackageJson(newVersion) {
  const content = JSON.parse(fs.readFileSync(PACKAGE_JSON, 'utf-8'));
  const oldVersion = content.version;
  content.version = newVersion;
  fs.writeFileSync(PACKAGE_JSON, JSON.stringify(content, null, 2) + '\n');
  return oldVersion;
}

/**
 * 更新 Cargo.toml
 */
function updateCargoToml(newVersion) {
  let content = fs.readFileSync(CARGO_TOML, 'utf-8');
  const match = content.match(/^version\s*=\s*"([^"]+)"/m);
  const oldVersion = match ? match[1] : null;
  content = content.replace(/^(version\s*=\s*)"[^"]+"/m, `$1"${newVersion}"`);
  fs.writeFileSync(CARGO_TOML, content);
  return oldVersion;
}

/**
 * 更新 tauri.conf.json
 */
function updateTauriConf(newVersion) {
  const content = JSON.parse(fs.readFileSync(TAURI_CONF, 'utf-8'));
  const oldVersion = content.version;
  content.version = newVersion;
  fs.writeFileSync(TAURI_CONF, JSON.stringify(content, null, 2) + '\n');
  return oldVersion;
}

/**
 * 创建发行说明模板
 */
function createReleaseNotes(version) {
  const filePath = path.join(RELEASE_DIR, `v${version}.md`);

  if (fs.existsSync(filePath)) {
    console.log(`  发行说明已存在: ${filePath}`);
    return false;
  }

  const template = `# StreamGrab v${version}

## ✨ 新增

- 功能描述

## 🚀 优化

- 优化项描述

## 🐛 修复

- 修复项描述

## 🔨 重构

- 重构项描述

## 📦 安装包

| 平台     | 文件                            |
| -------- | ------------------------------- |
| Windows  | \`StreamGrab_${version}_x64-setup.exe\` |
| macOS    | \`StreamGrab_${version}_x64.dmg\`        |
| Linux    | \`StreamGrab_${version}_amd64.AppImage\` |

## 致谢

感谢以下开源项目：
- [nilaoda](https://github.com/nilaoda) 开发的 [N_m3u8DL-RE](https://github.com/nilaoda/N_m3u8DL-RE) 流媒体下载引擎
- [FFmpeg](https://ffmpeg.org/) 多媒体处理框架
`;

  fs.writeFileSync(filePath, template);
  console.log(`  ✓ 创建发行说明: ${filePath}`);
  return true;
}

/**
 * 主函数
 */
function main() {
  const arg = process.argv[2];

  if (!arg) {
    console.log('用法: node scripts/release.js <version>');
    console.log('示例: node scripts/release.js 0.5.0');
    process.exit(1);
  }

  const newVersion = arg;

  // 验证版本号格式
  parseVersion(newVersion);

  // 读取当前版本
  const packageJson = JSON.parse(fs.readFileSync(PACKAGE_JSON, 'utf-8'));
  const currentVersion = packageJson.version;

  console.log(`\n🚀 发布准备: v${currentVersion} -> v${newVersion}\n`);

  // 1. 更新版本号
  console.log('📝 更新版本号:');
  updatePackageJson(newVersion);
  console.log('  ✓ package.json');

  updateCargoToml(newVersion);
  console.log('  ✓ src-tauri/Cargo.toml');

  updateTauriConf(newVersion);
  console.log('  ✓ src-tauri/tauri.conf.json');

  // 2. 创建发行说明
  console.log('\n📄 发行说明:');
  createReleaseNotes(newVersion);

  console.log(`\n✅ 发布准备完成！\n`);
  console.log('下一步:');
  console.log(`  1. 编辑发行说明: docs/releases/v${newVersion}.md`);
  console.log(`  2. 提交更改: git add -A && git commit -m "release: v${newVersion}"`);
  console.log(`  3. 创建标签: git tag -a v${newVersion} -m "v${newVersion}"`);
  console.log(`  4. 推送: git push origin main && git push origin v${newVersion}`);
}

main();
