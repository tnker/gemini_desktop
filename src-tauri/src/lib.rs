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
            // Geminiのヘッダーにコントロールを埋め込むアプローチ
            function injectWindowControls() {
                if (window !== window.top) return;
                if (document.getElementById('tauri-window-controls')) return;

                // Gemini のヘッダー（top-bar-actions）を特定する
                const topBar = document.querySelector('top-bar-actions');
                const rightSection = document.querySelector('top-bar-actions .right-section');
                if (!topBar || !rightSection) return; // 未レンダリングなら次回を待つ

                const btnContainer = document.createElement('div');
                btnContainer.id = 'tauri-window-controls';
                btnContainer.style.display = 'flex';
                btnContainer.style.alignItems = 'center';
                btnContainer.style.gap = '5px';
                btnContainer.style.height = '100%';
                btnContainer.style.marginLeft = '8px';
                btnContainer.style.marginRight = '8px';
                btnContainer.style.zIndex = '99999';

                const createBtn = (id, text) => {
                    const btn = document.createElement('button');
                    btn.id = id;
                    btn.textContent = text;
                    btn.style.width = '40px';
                    btn.style.height = '40px';
                    btn.style.background = 'transparent';
                    btn.style.border = 'none';
                    // WebUIのテーマ(ライト/ダーク)に合わせて文字色が変わることを期待し、CSS変数を使う場合もありますが、無難なグレーを採用
                    btn.style.color = '#888';
                    btn.style.cursor = 'pointer';
                    btn.style.display = 'flex';
                    btn.style.justifyContent = 'center';
                    btn.style.alignItems = 'center';
                    btn.style.fontSize = '12px';
                    btn.style.fontFamily = 'Segoe UI, sans-serif';
                    
                    btn.onmouseover = () => {
                        btn.style.background = 'rgba(128, 128, 128, 0.2)';
                        btn.style.color = '#fff';
                    };
                    btn.onmouseout = () => {
                        btn.style.background = 'transparent';
                        btn.style.color = '#888';
                    };
                    return btn;
                };

                const btnMin = createBtn('tb-min', '—');
                const btnMax = createBtn('tb-max', '☐');
                const btnClose = createBtn('tb-close', '✕');

                btnClose.onmouseover = () => {
                    btnClose.style.background = '#e81123';
                    btnClose.style.color = '#fff';
                };
                btnClose.onmouseout = () => {
                    btnClose.style.background = 'transparent';
                    btnClose.style.color = '#888';
                };

                const safeTauriCall = (action) => {
                    try {
                        if (!window.__TAURI__) return;
                        const win = window.__TAURI__.window 
                            ? window.__TAURI__.window.getCurrentWindow() 
                            : window.__TAURI__.core.Window.getCurrent();
                        win[action]();
                    } catch (err) {
                        console.error('Tauri executing error:', err);
                    }
                };

                btnMin.onclick = () => safeTauriCall('minimize');
                btnMax.onclick = () => safeTauriCall('toggleMaximize');
                btnClose.onclick = () => safeTauriCall('close');

                btnContainer.appendChild(btnMin);
                btnContainer.appendChild(btnMax);
                btnContainer.appendChild(btnClose);

                rightSection.appendChild(btnContainer);

                // ドラッグ領域の実装（top-bar-actions全体に対するフック）
                topBar.addEventListener('mousedown', (e) => {
                    if (e.button !== 0) return; // 左クリックのみ
                    
                    // インタラクティブな要素（ボタン、リンク、入力など）をクリックした場合はドラッグを発火させない
                    const isInteractive = e.target.closest('button, a, input, textarea, [role="button"], [contenteditable], svg');
                    if (isInteractive) return;

                    try {
                        if (!window.__TAURI__) return;
                        const win = window.__TAURI__.window 
                            ? window.__TAURI__.window.getCurrentWindow() 
                            : window.__TAURI__.core.Window.getCurrent();
                        win.startDragging();
                    } catch(err) {
                        console.error('Drag error:', err);
                    }
                });
            }

            if (window === window.top) {
                // DOMが遅延で描画されるケース（SPAのロード待ち）に対応するためポーリング
                setInterval(injectWindowControls, 1000);
            }
            "#;

            let url = "https://gemini.google.com/app".parse().unwrap();
            let mut builder = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(url))
                .title("Gemini Desktop")
                .inner_size(800.0, 600.0)
                .decorations(false)
                .visible(false) // 状態復元まで隠しておく（フラッシュ防止）
                .initialization_script(script);

            // コマンドライン引数を取得し、初期入力テキストとして利用する
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                let initial_query = args[1..].join(" ");
                // 安全にJS文字列化
                if let Ok(js_query) = serde_json::to_string(&initial_query) {
                    let auto_input_script = format!(r#"
                        window.addEventListener('DOMContentLoaded', () => {{
                            const targetText = {};
                            if (!targetText) return;
                            
                            const tryInject = () => {{
                                // Geminiのチャット入力欄
                                const editor = document.querySelector('rich-textarea div[contenteditable="true"], div[contenteditable="true"][role="textbox"]');
                                // 見つかり、かつまだ入力されていない場合
                                if (editor && editor.textContent.trim() === '') {{
                                    editor.focus();
                                    document.execCommand('insertText', false, targetText);
                                    return true;
                                }}
                                return false;
                            }};

                            const int = setInterval(() => {{
                                if (tryInject()) clearInterval(int);
                            }}, 500);
                            
                            setTimeout(() => clearInterval(int), 15000);
                        }});
                    "#, js_query);
                    builder = builder.initialization_script(&auto_input_script);
                }
            }

            let window = builder.build().unwrap();

            // ウィンドウ生成後に状態を復元（サイズ、位置など）
            use tauri_plugin_window_state::{WindowExt, StateFlags};
            let _ = window.restore_state(StateFlags::all());
            let _ = window.show();

            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // 他のインスタンスが起動されたときに呼ばれる（自分自身がプライマリインスタンス）
            if let Some(window) = app.get_webview_window("main") {
                // ウィンドウを前面に出す
                let _ = window.unminimize();
                let _ = window.set_focus();

                // 引数があればGeminiのチャット欄に入力する
                if args.len() > 1 {
                    let initial_query = args[1..].join(" ");
                    if let Ok(js_query) = serde_json::to_string(&initial_query) {
                        let eval_script = format!(r#"
                            (function() {{
                                const targetText = {};
                                if (!targetText) return;
                                
                                const tryInject = () => {{
                                    const editor = document.querySelector('rich-textarea div[contenteditable="true"], div[contenteditable="true"][role="textbox"]');
                                    // 既に何らかのテキストがある場合は追記、またはクリアしてから入力するかですが、挙動上一旦そのまま execCommand します
                                    if (editor) {{
                                        editor.focus();
                                        document.execCommand('insertText', false, targetText);
                                        return true;
                                    }}
                                    return false;
                                }};

                                if (!tryInject()) {{
                                    const int = setInterval(() => {{
                                        if (tryInject()) clearInterval(int);
                                    }}, 500);
                                    setTimeout(() => clearInterval(int), 10000);
                                }}
                            }})();
                        "#, js_query);
                        let _ = window.eval(&eval_script);
                    }
                }
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
