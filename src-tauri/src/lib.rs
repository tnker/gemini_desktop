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
            let win = app.get_webview_window("main").unwrap();
            let script = r#"
            window.addEventListener('DOMContentLoaded', () => {
                if(document.getElementById('tauri-custom-titlebar')) return;
                const bar = document.createElement('div');
                bar.id = 'tauri-custom-titlebar';
                bar.innerHTML = `
                    <div style="flex-grow: 1; padding-left: 16px; font-size: 14px; font-family: sans-serif; display: flex; align-items: center;" data-tauri-drag-region>Gemini Desktop</div>
                    <div style="display: flex;">
                        <button id="tb-min" style="width: 46px; background: transparent; border: none; color: #e3e3e3; cursor: pointer; display: flex; justify-content: center; align-items: center;">—</button>
                        <button id="tb-max" style="width: 46px; background: transparent; border: none; color: #e3e3e3; cursor: pointer; display: flex; justify-content: center; align-items: center;">☐</button>
                        <button id="tb-close" style="width: 46px; background: transparent; border: none; color: #e3e3e3; cursor: pointer; display: flex; justify-content: center; align-items: center;">✕</button>
                    </div>
                `;
                bar.style.position = 'fixed';
                bar.style.top = '0';
                bar.style.left = '0';
                bar.style.right = '0';
                bar.style.height = '40px';
                bar.style.background = '#202124';
                bar.style.color = '#e3e3e3';
                bar.style.display = 'flex';
                bar.style.justifyContent = 'space-between';
                bar.style.zIndex = '99999999';
                bar.style.userSelect = 'none';
                bar.style.borderBottom = '1px solid #3c4043';
                bar.setAttribute('data-tauri-drag-region', 'true');
                document.body.prepend(bar);
                // Geminiの元のヘッダーが隠れないようにpaddingを追加
                document.body.style.paddingTop = '40px';

                document.getElementById('tb-min').onclick = () => window.__TAURI__.window.getCurrentWindow().minimize();
                document.getElementById('tb-max').onclick = () => window.__TAURI__.window.getCurrentWindow().toggleMaximize();
                document.getElementById('tb-close').onclick = () => window.__TAURI__.window.getCurrentWindow().close();
                
                // ホバーエフェクトの追加
                ['tb-min', 'tb-max', 'tb-close'].forEach(id => {
                    const el = document.getElementById(id);
                    el.onmouseover = () => el.style.background = 'rgba(255, 255, 255, 0.1)';
                    el.onmouseout = () => el.style.background = 'transparent';
                    el.onmousedown = () => el.style.background = 'rgba(255, 255, 255, 0.2)';
                    el.onmouseup = () => el.style.background = 'rgba(255, 255, 255, 0.1)';
                });
            });
            "#;
            // Initialization script applies to all navigations
            win.eval(script).unwrap();
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
