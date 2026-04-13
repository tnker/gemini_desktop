import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Webview } from "@tauri-apps/api/webview";
import "./App.css";

function App() {
  const appWindow = getCurrentWindow();
  const [errorMsg, setErrorMsg] = useState<string>("");

  useEffect(() => {
    const createWebview = async () => {
      try {
        // 既存のWebviewがあれば作成しない
        const existing = await Webview.getByLabel('gemini-view');
        if (existing) return;

        const webview = new Webview(appWindow, 'gemini-view', {
          url: 'https://gemini.google.com/app',
          x: 0,
          y: 40,
          width: window.innerWidth,
          height: window.innerHeight - 40,
        });

        // Event listener for resize
        appWindow.listen('tauri://resize', async () => {
          try {
            const size = await appWindow.innerSize();
            await webview.setSize({
              type: "Logical", // 'Physical' だと倍率でおかしくなる場合があるためとりあえずLogical
              width: window.innerWidth,
              height: window.innerHeight - 40,
            });
          } catch(e) {
            // Resize error
          }
        });
      } catch (e: any) {
        setErrorMsg(e.toString());
      }
    };

    createWebview();
  }, [appWindow]);

  return (
    <div className="container">
      <div data-tauri-drag-region className="titlebar">
        <div className="titlebar-title" data-tauri-drag-region>Gemini Desktop</div>
        <div className="titlebar-buttons">
          <button onClick={() => appWindow.minimize()}>—</button>
          <button onClick={() => appWindow.toggleMaximize()}>☐</button>
          <button onClick={() => appWindow.close()}>✕</button>
        </div>
      </div>

      <div className="webview-placeholder">
        {errorMsg ? (
          <p style={{ color: "red" }}>Error: {errorMsg}</p>
        ) : (
          <p>Loading Gemini...</p>
        )}
      </div>
    </div>
  );
}

export default App;
