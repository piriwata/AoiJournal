import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import NaturalLanguageInput from "./components/NaturalLanguageInput";
import TransactionList from "./components/TransactionList";
import ReportView from "./components/ReportView";
import Settings from "./components/Settings";

type Tab = "transactions" | "ledger" | "pl" | "bs" | "settings";

interface Journal {
  meta: {
    business_name: string;
    fiscal_year_start: string;
    fiscal_year_end: string;
  };
  accounts: Account[];
  transactions: Transaction[];
}

interface Account {
  name: string;
  account_type: string;
}

interface Transaction {
  id: string;
  date: string;
  narration: string;
  postings: Posting[];
  memo?: string;
}

interface Posting {
  account: string;
  amount: number;
}

export type { Journal, Account, Transaction, Posting };

function App() {
  const [journal, setJournal] = useState<Journal | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("transactions");
  const [filePath, setFilePath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showNewDialog, setShowNewDialog] = useState(false);

  useEffect(() => {
    invoke<Journal | null>("get_journal").then((j) => {
      if (j) setJournal(j);
    });
  }, []);

  const handleNew = useCallback(async () => {
    setShowNewDialog(true);
  }, []);

  const handleOpen = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Journal", extensions: ["journal", "txt"] }],
      });
      if (selected) {
        const { readTextFile } = await import("@tauri-apps/plugin-fs");
        const content = await readTextFile(selected as string);
        const j = await invoke<Journal>("open_journal_from_content", {
          content,
          path: selected as string,
        });
        setJournal(j);
        setFilePath(selected as string);
        setError(null);
      }
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const handleSave = useCallback(async () => {
    try {
      const content = await invoke<string>("save_journal");
      let targetPath = filePath;
      if (!targetPath) {
        targetPath = await save({
          filters: [{ name: "Journal", extensions: ["journal"] }],
          defaultPath: "journal.journal",
        });
        if (!targetPath) return;
        setFilePath(targetPath);
      }
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");
      await writeTextFile(targetPath, content);
    } catch (e) {
      setError(String(e));
    }
  }, [filePath]);

  const refreshJournal = useCallback(async () => {
    const j = await invoke<Journal | null>("get_journal");
    if (j) setJournal(j);
  }, []);

  const tabs: { id: Tab; label: string }[] = [
    { id: "transactions", label: "取引" },
    { id: "ledger", label: "総勘定元帳" },
    { id: "pl", label: "損益計算書" },
    { id: "bs", label: "貸借対照表" },
    { id: "settings", label: "設定" },
  ];

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-left">
          <h1 className="app-title">
            <span className="app-title-blue">青</span>色 Journal
          </h1>
          {journal && (
            <span className="business-name">{journal.meta.business_name}</span>
          )}
        </div>
        <div className="header-actions">
          <button className="btn btn-secondary" onClick={handleNew}>
            新規
          </button>
          <button className="btn btn-secondary" onClick={handleOpen}>
            開く
          </button>
          <button
            className="btn btn-primary"
            onClick={handleSave}
            disabled={!journal}
          >
            保存
          </button>
        </div>
      </header>

      {error && (
        <div className="error-banner">
          <span>{error}</span>
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      {showNewDialog && (
        <NewJournalDialog
          onClose={() => setShowNewDialog(false)}
          onCreate={async (name, start, end) => {
            try {
              const j = await invoke<Journal>("new_journal", {
                businessName: name,
                fiscalYearStart: start,
                fiscalYearEnd: end,
              });
              setJournal(j);
              setFilePath(null);
              setShowNewDialog(false);
            } catch (e) {
              setError(String(e));
            }
          }}
        />
      )}

      <nav className="tab-nav">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`tab-btn ${activeTab === tab.id ? "active" : ""}`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </nav>

      <main className="app-main">
        {!journal ? (
          <div className="welcome">
            <h2>Aoi Journal へようこそ</h2>
            <p>青色申告のための複式簿記ツール</p>
            <div className="welcome-actions">
              <button className="btn btn-primary btn-lg" onClick={handleNew}>
                新規帳簿を作成
              </button>
              <button className="btn btn-secondary btn-lg" onClick={handleOpen}>
                既存の帳簿を開く
              </button>
            </div>
          </div>
        ) : activeTab === "transactions" ? (
          <div className="tab-content">
            <NaturalLanguageInput
              journal={journal}
              onTransactionAdded={refreshJournal}
            />
            <TransactionList
              journal={journal}
              onRefresh={refreshJournal}
            />
          </div>
        ) : activeTab === "settings" ? (
          <Settings />
        ) : (
          <ReportView reportType={activeTab} />
        )}
      </main>
    </div>
  );
}

function NewJournalDialog({
  onClose,
  onCreate,
}: {
  onClose: () => void;
  onCreate: (name: string, start: string, end: string) => void;
}) {
  const [name, setName] = useState("");
  const [start, setStart] = useState(new Date().getFullYear() + "-01-01");
  const [end, setEnd] = useState(new Date().getFullYear() + "-12-31");

  return (
    <div className="modal-overlay">
      <div className="modal">
        <h2>新規帳簿の作成</h2>
        <div className="form-group">
          <label>屋号・事業所名</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="例：山田太郎 フリーランス"
          />
        </div>
        <div className="form-group">
          <label>事業年度 開始日</label>
          <input
            type="date"
            value={start}
            onChange={(e) => setStart(e.target.value)}
          />
        </div>
        <div className="form-group">
          <label>事業年度 終了日</label>
          <input
            type="date"
            value={end}
            onChange={(e) => setEnd(e.target.value)}
          />
        </div>
        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            キャンセル
          </button>
          <button
            className="btn btn-primary"
            onClick={() => onCreate(name, start, end)}
            disabled={!name}
          >
            作成
          </button>
        </div>
      </div>
    </div>
  );
}

export default App;
