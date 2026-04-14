use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let script = r#"
            function injectTitlebar() {
                if (!document.body) return;
                if (document.getElementById('tauri-custom-titlebar')) return;
                
                const bar = document.createElement('div');
                bar.id = 'tauri-custom-titlebar';
                bar.style.position = 'fixed';
                bar.style.top = '0';
                bar.style.left = '0';
                bar.style.right = '0';
                bar.style.height = '40px';
                bar.style.color = '#e3e3e3';
                bar.style.display = 'flex';
                bar.style.justifyContent = 'space-between';
                bar.style.zIndex = '99999999';
                bar.style.userSelect = 'none';

                // 背景色をウィンドウ幅に応じて切り替える処理
                const updateTitlebarBg = () => {
                    if (window.innerWidth >= 960) {
                        bar.style.background = '#1e1f20';
                    } else {
                        bar.style.background = 'transparent';
                    }
                };
                // 初回実行と画面リサイズ時のイベント登録
                updateTitlebarBg();
                window.addEventListener('resize', updateTitlebarBg);

                const dragRegion = document.createElement('div');
                dragRegion.id = 'tb-drag';
                dragRegion.textContent = '';
                dragRegion.style.flexGrow = '1';
                dragRegion.style.paddingLeft = '16px';
                dragRegion.style.fontSize = '14px';
                dragRegion.style.fontFamily = 'sans-serif';
                dragRegion.style.display = 'flex';
                dragRegion.style.alignItems = 'center';
                dragRegion.style.cursor = 'grab';

                const btnContainer = document.createElement('div');
                btnContainer.style.display = 'flex';

                const createBtn = (id, text) => {
                    const btn = document.createElement('button');
                    btn.id = id;
                    btn.textContent = text;
                    btn.style.width = '46px';
                    btn.style.background = 'transparent';
                    btn.style.border = 'none';
                    btn.style.color = '#e3e3e3';
                    btn.style.cursor = 'pointer';
                    btn.style.display = 'flex';
                    btn.style.justifyContent = 'center';
                    btn.style.alignItems = 'center';
                    return btn;
                };

                const btnMin = createBtn('tb-min', '—');
                const btnMax = createBtn('tb-max', '☐');
                const btnClose = createBtn('tb-close', '✕');

                btnContainer.appendChild(btnMin);
                btnContainer.appendChild(btnMax);
                btnContainer.appendChild(btnClose);

                bar.appendChild(dragRegion);
                bar.appendChild(btnContainer);

                // titlebar自体は<body>の中ではなく、<html>直下に配置して、Gemini全体レイヤーと分離する
                if (document.documentElement) {
                    document.documentElement.appendChild(bar);
                }

                // --------- ここからオーバーラップ（被り）の解消処理 --------- //
                // GeminiはSPAで position: fixed のヘッダーを持っているため、単なる margin-top ではヘッダーが下がりません。
                // そこで、<body> に transform を付与して固定要素の座標起点をずらし、全体を40px下げます。
                const enforceLayout = () => {
                    if (document.body) {
                        // transformをかけてbodyを「新しい固定要素の起点（Containing Block）」に変更する
                        document.body.style.setProperty('transform', 'translateZ(0)', 'important');
                        // 40px下に押し下げる
                        document.body.style.setProperty('margin-top', '40px', 'important');
                        // はみ出ないように高さを微調整
                        document.body.style.setProperty('height', 'calc(100vh - 40px)', 'important');
                        // 念のため固定要素である場合はabsoluteに変える
                        if (getComputedStyle(document.body).position === 'fixed') {
                            document.body.style.setProperty('position', 'absolute', 'important');
                        }
                    }
                };
                
                // 初回実行
                enforceLayout();
                
                // SPAの画面更新でbodyが差し替えられたりスタイルがリセットされた際にも再適用する
                setInterval(enforceLayout, 500);
                // ------------------------------------------------------------- //

                // ドラッグ処理を直接マニュアルで実行する
                dragRegion.addEventListener('mousedown', (e) => {
                    // 左クリックのみ
                    if (e.button === 0) {
                        try {
                            if (!window.__TAURI__) return;
                            const win = window.__TAURI__.window ? window.__TAURI__.window.getCurrentWindow() : window.__TAURI__.core.Window.getCurrent();
                            win.startDragging();
                        } catch(err) {
                            console.error('Drag error:', err);
                        }
                    }
                });

                // 最小化・最大化・閉じるボタンの挙動を安全に実行し、エラーをコンソールに出す
                const safeTauriCall = (action) => {
                    try {
                        if (!window.__TAURI__) {
                            console.error("Tauri API is not injected. Check withGlobalTauri in tauri.conf.json");
                            return;
                        }
                        const win = window.__TAURI__.window ? window.__TAURI__.window.getCurrentWindow() : window.__TAURI__.core.Window.getCurrent();
                        win[action]();
                    } catch (err) {
                        console.error(`Error executing ${action}:`, err, 'TAURI API:', window.__TAURI__);
                    }
                };

                btnMin.onclick = () => safeTauriCall('minimize');
                btnMax.onclick = () => safeTauriCall('toggleMaximize');
                btnClose.onclick = () => safeTauriCall('close');
                
                // ホバーエフェクトの追加
                [btnMin, btnMax, btnClose].forEach(el => {
                    el.onmouseover = () => el.style.background = 'rgba(255, 255, 255, 0.1)';
                    el.onmouseout = () => el.style.background = 'transparent';
                    el.onmousedown = () => el.style.background = 'rgba(255, 255, 255, 0.2)';
                    el.onmouseup = () => el.style.background = 'rgba(255, 255, 255, 0.1)';
                });
            }

            // ページロード時や、SPAでDOMが書き換えられた後にも対応できるように定期監視
            setInterval(injectTitlebar, 500);
            window.addEventListener('DOMContentLoaded', injectTitlebar);
            "#;

            let url = "https://gemini.google.com/app".parse().unwrap();
            let window = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
                .title("Gemini Desktop")
                .inner_size(800.0, 600.0)
                .decorations(false)
                .visible(false) // 状態復元まで隠しておく（フラッシュ防止）
                .initialization_script(script)
                .build()
                .unwrap();

            // ウィンドウ生成後に状態を復元（サイズ、位置など）
            use tauri_plugin_window_state::{WindowExt, StateFlags};
            let _ = window.restore_state(StateFlags::all());
            let _ = window.show();

            Ok(())
        })
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
