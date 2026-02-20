#!/usr/bin/env node

/**
 * 版本号同步脚本
 *
 * 以 package.json 为单一真相来源，同步版本号到：
 * - src-tauri/Cargo.toml
 * - src-tauri/tauri.conf.json
 *
 * 用法：
 *   node scripts/bump-version.js <version>
 *   node scripts/bump-version.js 0.5.0
 *   node scripts/bump-version.js patch  # 0.4.0 -> 0.4.1
 *   node scripts/bump-version.js minor  # 0.4.0 -> 0.5.0
 *   node scripts/bump-version.js major  # 0.4.0 -> 1.0.0
 */

const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.join(__dirname, '..');
const PACKAGE_JSON = path.join(ROOT_DIR, 'package.json');
const CARGO_TOML = path.join(ROOT_DIR, 'src-tauri', 'Cargo.toml');
const TAURI_CONF = path.join(ROOT_DIR, 'src-tauri', 'tauri.conf.json');

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
  };
}

/**
 * 版本号递增
 */
function bumpVersion(currentVersion, type) {
  const v = parseVersion(currentVersion);
  switch (type) {
    case 'major':
      return `${v.major + 1}.0.0`;
    case 'minor':
      return `${v.major}.${v.minor + 1}.0`;
    case 'patch':
      return `${v.major}.${v.minor}.${v.patch + 1}`;
    default:
      return type; // 直接使用传入的版本号
  }
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
 * 主函数
 */
function main() {
  const arg = process.argv[2];

  if (!arg) {
    console.log('用法: node scripts/bump-version.js <version|patch|minor|major>');
    console.log('示例:');
    console.log('  node scripts/bump-version.js 0.5.0');
    console.log('  node scripts/bump-version.js patch');
    console.log('  node scripts/bump-version.js minor');
    console.log('  node scripts/bump-version.js major');
    process.exit(1);
  }

  // 读取当前版本
  const packageJson = JSON.parse(fs.readFileSync(PACKAGE_JSON, 'utf-8'));
  const currentVersion = packageJson.version;

  // 计算新版本
  const newVersion = bumpVersion(currentVersion, arg);

  if (newVersion === currentVersion) {
    console.log(`版本号已是 ${newVersion}，无需更新`);
    process.exit(0);
  }

  console.log(`版本号更新: ${currentVersion} -> ${newVersion}`);

  // 更新所有文件
  updatePackageJson(newVersion);
  console.log('  ✓ package.json');

  updateCargoToml(newVersion);
  console.log('  ✓ src-tauri/Cargo.toml');

  updateTauriConf(newVersion);
  console.log('  ✓ src-tauri/tauri.conf.json');

  console.log('\n版本号同步完成！');
  console.log('\n下一步:');
  console.log('  1. 创建发行说明: docs/releases/v' + newVersion + '.md');
  console.log('  2. 提交更改: git add -A && git commit -m "chore: bump version to v' + newVersion + '"');
  console.log('  3. 创建标签: git tag -a v' + newVersion + ' -m "v' + newVersion + '"');
}

main();
