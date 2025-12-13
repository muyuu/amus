import { execSync } from 'child_process';

const isCI = (process.env.CI ?? 'false') === 'true';

function exec(cmd, desc) {
  console.log(`📦 ${desc}...`);
  try {
    execSync(cmd, { stdio: 'inherit' });
    console.log(`✅ ${desc}完了`);
  } catch {
    console.log(`⚠️ ${desc}失敗`);
  }
}

console.log('🚀 セットアップ開始');

console.log(`ℹ️ ${isCI ? 'CI' : 'ユーザー'}環境向けセットアップを実行しています`);
if (!isCI) {
  exec('cargo install cargo-watch', 'cargo-watch');
  exec('cargo install miniserve', 'miniserve');
}
exec('cargo install wasm-pack', 'wasm-pack');
exec('rustup target add wasm32-unknown-unknown', 'WASMターゲット追加');
exec('npm install', 'npmパッケージ');

console.log('🎉 完了！');
