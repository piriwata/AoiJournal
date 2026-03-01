import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Journal, Posting } from "../App";

interface ProposedTransaction {
  date: string;
  narration: string;
  postings: Posting[];
  notes?: string;
}

interface Props {
  journal: Journal;
  onTransactionAdded: () => void;
}

export default function NaturalLanguageInput({ journal, onTransactionAdded }: Props) {
  const [text, setText] = useState("");
  const [proposals, setProposals] = useState<ProposedTransaction[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [manualMode, setManualMode] = useState(false);

  const [manualDate, setManualDate] = useState(new Date().toISOString().slice(0, 10));
  const [manualNarration, setManualNarration] = useState("");
  const [manualPostings, setManualPostings] = useState<Posting[]>([
    { account: "", amount: 0 },
    { account: "", amount: 0 },
  ]);

  const handlePropose = async () => {
    if (!text.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ProposedTransaction[]>("propose_transaction_from_nlp", {
        text,
      });
      setProposals(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleConfirm = async (proposal: ProposedTransaction) => {
    try {
      await invoke("add_transaction", {
        date: proposal.date,
        narration: proposal.narration,
        postings: proposal.postings,
        memo: proposal.notes,
      });
      setProposals([]);
      setText("");
      onTransactionAdded();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleManualSave = async () => {
    try {
      await invoke("add_transaction", {
        date: manualDate,
        narration: manualNarration,
        postings: manualPostings.filter((p) => p.account && p.amount !== 0),
        memo: undefined,
      });
      setManualNarration("");
      setManualPostings([
        { account: "", amount: 0 },
        { account: "", amount: 0 },
      ]);
      onTransactionAdded();
    } catch (e) {
      setError(String(e));
    }
  };

  const updatePosting = (index: number, field: keyof Posting, value: string | number) => {
    const updated = [...manualPostings];
    updated[index] = { ...updated[index], [field]: value };
    setManualPostings(updated);
  };

  const manualTotal = manualPostings.reduce((sum, p) => sum + (Number(p.amount) || 0), 0);

  return (
    <div className="nl-input-section">
      <div className="section-header">
        <h2>取引の入力</h2>
        <div className="input-mode-toggle">
          <button
            className={`mode-btn ${!manualMode ? "active" : ""}`}
            onClick={() => setManualMode(false)}
          >
            AI入力
          </button>
          <button
            className={`mode-btn ${manualMode ? "active" : ""}`}
            onClick={() => setManualMode(true)}
          >
            手動入力
          </button>
        </div>
      </div>

      {!manualMode ? (
        <div className="ai-input">
          <div className="nl-input-row">
            <textarea
              className="nl-textarea"
              value={text}
              onChange={(e) => setText(e.target.value)}
              placeholder="例：3月1日 AWSの費用 12,345円 クレジットカード払い"
              rows={2}
              onKeyDown={(e) => {
                if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
                  handlePropose();
                }
              }}
            />
            <button
              className="btn btn-primary"
              onClick={handlePropose}
              disabled={loading || !text.trim()}
            >
              {loading ? "処理中..." : "提案する"}
            </button>
          </div>
          <p className="hint">Ctrl+Enter で提案 / Ollama が必要です</p>

          {error && <div className="error-message">{error}</div>}

          {proposals.map((proposal, idx) => (
            <ProposalCard
              key={idx}
              proposal={proposal}
              accounts={journal.accounts.map((a) => a.name)}
              onConfirm={() => handleConfirm(proposal)}
              onDiscard={() => setProposals((p) => p.filter((_, i) => i !== idx))}
            />
          ))}
        </div>
      ) : (
        <div className="manual-input">
          <div className="form-row">
            <div className="form-group">
              <label>日付</label>
              <input
                type="date"
                value={manualDate}
                onChange={(e) => setManualDate(e.target.value)}
              />
            </div>
            <div className="form-group flex-grow">
              <label>摘要</label>
              <input
                type="text"
                value={manualNarration}
                onChange={(e) => setManualNarration(e.target.value)}
                placeholder="例：売上入金"
              />
            </div>
          </div>

          <div className="postings-table">
            <div className="postings-header">
              <span>勘定科目</span>
              <span>金額 (JPY)</span>
            </div>
            {manualPostings.map((posting, idx) => (
              <div key={idx} className="posting-row">
                <select
                  value={posting.account}
                  onChange={(e) => updatePosting(idx, "account", e.target.value)}
                >
                  <option value="">-- 勘定科目を選択 --</option>
                  {journal.accounts.map((a) => (
                    <option key={a.name} value={a.name}>
                      {a.name}
                    </option>
                  ))}
                </select>
                <input
                  type="number"
                  value={posting.amount || ""}
                  onChange={(e) => updatePosting(idx, "amount", parseInt(e.target.value) || 0)}
                  placeholder="金額"
                />
                {manualPostings.length > 2 && (
                  <button
                    className="btn-icon"
                    onClick={() =>
                      setManualPostings((p) => p.filter((_, i) => i !== idx))
                    }
                  >
                    ×
                  </button>
                )}
              </div>
            ))}
            <button
              className="btn btn-ghost"
              onClick={() =>
                setManualPostings((p) => [...p, { account: "", amount: 0 }])
              }
            >
              + 行を追加
            </button>
          </div>

          <div className={`balance-indicator ${manualTotal === 0 ? "balanced" : "unbalanced"}`}>
            合計: {manualTotal.toLocaleString()} JPY{" "}
            {manualTotal === 0 ? "✓ バランス" : "⚠ 不一致"}
          </div>

          {error && <div className="error-message">{error}</div>}

          <button
            className="btn btn-primary"
            onClick={handleManualSave}
            disabled={manualTotal !== 0 || !manualNarration}
          >
            仕訳を登録
          </button>
        </div>
      )}
    </div>
  );
}

function ProposalCard({
  proposal,
  accounts,
  onConfirm,
  onDiscard,
}: {
  proposal: ProposedTransaction;
  accounts: string[];
  onConfirm: () => void;
  onDiscard: () => void;
}) {
  const [editedProposal, setEditedProposal] = useState(proposal);
  const total = editedProposal.postings.reduce((sum, p) => sum + p.amount, 0);

  return (
    <div className="proposal-card">
      <div className="proposal-header">
        <span className="proposal-date">{editedProposal.date}</span>
        <span className="proposal-narration">{editedProposal.narration}</span>
        {editedProposal.notes && (
          <span className="proposal-notes">{editedProposal.notes}</span>
        )}
      </div>
      <div className="postings-table">
        {editedProposal.postings.map((posting, idx) => (
          <div key={idx} className="posting-row">
            <select
              value={posting.account}
              onChange={(e) => {
                const updated = [...editedProposal.postings];
                updated[idx] = { ...updated[idx], account: e.target.value };
                setEditedProposal({ ...editedProposal, postings: updated });
              }}
            >
              <option value={posting.account}>{posting.account}</option>
              {accounts
                .filter((a) => a !== posting.account)
                .map((a) => (
                  <option key={a} value={a}>
                    {a}
                  </option>
                ))}
            </select>
            <input
              type="number"
              value={posting.amount}
              onChange={(e) => {
                const updated = [...editedProposal.postings];
                updated[idx] = {
                  ...updated[idx],
                  amount: parseInt(e.target.value) || 0,
                };
                setEditedProposal({ ...editedProposal, postings: updated });
              }}
            />
          </div>
        ))}
      </div>
      <div className={`balance-indicator ${total === 0 ? "balanced" : "unbalanced"}`}>
        合計: {total.toLocaleString()} JPY {total === 0 ? "✓" : "⚠ 不一致"}
      </div>
      <div className="proposal-actions">
        <button className="btn btn-secondary" onClick={onDiscard}>
          破棄
        </button>
        <button
          className="btn btn-primary"
          onClick={onConfirm}
          disabled={total !== 0}
        >
          ✓ 登録する
        </button>
      </div>
    </div>
  );
}
