import init, { start } from './pkg/amus.js';

async function run() {
    // WASMモジュールを初期化
    await init();

    // canvas要素を作成
    const canvas = document.createElement('canvas');
    canvas.id = 'canvas';
    // canvas要素にサイズを設定
    canvas.style.width = '100%';
    canvas.style.height = '100%';
    canvas.width = 1200;
    canvas.height = 840;

    // rootにcanvasを追加
    const root = document.getElementById('root');
    if (root) {
        // root要素にもサイズを設定
        root.style.width = '100%';
        root.style.height = '100vh';
        root.style.margin = '0';
        root.style.padding = '0';

        root.appendChild(canvas);

        // WASMアプリを起動
        try {
            console.log('Starting WASM app...');
            await start('canvas');
            console.log('WASM app started successfully');
        } catch (error) {
            console.error('Failed to start WASM app:', error);
            console.error('Error details:', error.toString());
        }
    } else {
        console.error('root element not found');
    }
}

run();

