import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface OllamaConfig {
  base_url: string;
  model: string;
  timeout_secs: number;
}

export default function Settings() {
  const [config, setConfig] = useState<OllamaConfig>({
    base_url: "http://localhost:11434",
    model: "llama3.2",
    timeout_secs: 60,
  });
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<OllamaConfig>("get_ollama_config").then(setConfig).catch(() => {});
  }, []);

  const handleSave = async () => {
    try {
      await invoke("set_ollama_config", {
        baseUrl: config.base_url,
        model: config.model,
        timeoutSecs: config.timeout_secs,
      });
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="settings-view">
      <h2>設定</h2>
      <section className="settings-section">
        <h3>Ollama 設定</h3>
        <p className="hint">ローカルLLMによるAI入力支援の設定です。Ollamaが不要な場合は無視してください。</p>
        <div className="form-group">
          <label>Ollama URL</label>
          <input
            type="text"
            value={config.base_url}
            onChange={(e) => setConfig({ ...config, base_url: e.target.value })}
          />
        </div>
        <div className="form-group">
          <label>モデル名</label>
          <input
            type="text"
            value={config.model}
            onChange={(e) => setConfig({ ...config, model: e.target.value })}
            placeholder="例：llama3.2, qwen2.5"
          />
        </div>
        <div className="form-group">
          <label>タイムアウト（秒）</label>
          <input
            type="number"
            value={config.timeout_secs}
            onChange={(e) =>
              setConfig({ ...config, timeout_secs: parseInt(e.target.value) || 60 })
            }
          />
        </div>
        {error && <div className="error-message">{error}</div>}
        {saved && <div className="success-message">保存しました</div>}
        <button className="btn btn-primary" onClick={handleSave}>
          保存
        </button>
      </section>
    </div>
  );
}
