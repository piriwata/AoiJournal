import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Journal, Transaction, Posting } from "../App";

interface Props {
  journal: Journal;
  onRefresh: () => void;
}

export default function TransactionList({ journal, onRefresh }: Props) {
  const [search, setSearch] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const filteredTxs = journal.transactions
    .filter(
      (tx) =>
        tx.narration.includes(search) ||
        tx.postings.some((p) => p.account.includes(search))
    )
    .sort((a, b) => b.date.localeCompare(a.date));

  const handleDelete = async (id: string) => {
    if (!confirm("この仕訳を削除しますか？")) return;
    try {
      await invoke("delete_transaction", { id });
      onRefresh();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="transaction-list-section">
      <div className="section-header">
        <h2>仕訳一覧 ({filteredTxs.length}件)</h2>
        <input
          type="search"
          className="search-input"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="摘要・勘定科目で検索"
        />
      </div>

      {error && <div className="error-message">{error}</div>}

      {filteredTxs.length === 0 ? (
        <div className="empty-state">仕訳がありません</div>
      ) : (
        <table className="tx-table">
          <thead>
            <tr>
              <th>日付</th>
              <th>摘要</th>
              <th>借方</th>
              <th>貸方</th>
              <th>金額</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {filteredTxs.map((tx) =>
              editingId === tx.id ? (
                <EditRow
                  key={tx.id}
                  tx={tx}
                  accounts={journal.accounts.map((a) => a.name)}
                  onSave={async (updated) => {
                    try {
                      await invoke("update_transaction", {
                        id: tx.id,
                        date: updated.date,
                        narration: updated.narration,
                        postings: updated.postings,
                        memo: updated.memo,
                      });
                      setEditingId(null);
                      onRefresh();
                    } catch (e) {
                      setError(String(e));
                    }
                  }}
                  onCancel={() => setEditingId(null)}
                />
              ) : (
                <TransactionRow
                  key={tx.id}
                  tx={tx}
                  onEdit={() => setEditingId(tx.id)}
                  onDelete={() => handleDelete(tx.id)}
                />
              )
            )}
          </tbody>
        </table>
      )}
    </div>
  );
}

function TransactionRow({
  tx,
  onEdit,
  onDelete,
}: {
  tx: Transaction;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const debits = tx.postings.filter((p) => p.amount > 0);
  const credits = tx.postings.filter((p) => p.amount < 0);
  const amount = debits.reduce((s, p) => s + p.amount, 0);

  return (
    <tr className="tx-row">
      <td>{tx.date}</td>
      <td>{tx.narration}</td>
      <td>{debits.map((p) => p.account).join(", ")}</td>
      <td>{credits.map((p) => p.account).join(", ")}</td>
      <td className="amount">{amount.toLocaleString()}</td>
      <td className="row-actions">
        <button className="btn-icon" onClick={onEdit} title="編集">
          ✎
        </button>
        <button className="btn-icon btn-danger" onClick={onDelete} title="削除">
          ✕
        </button>
      </td>
    </tr>
  );
}

function EditRow({
  tx,
  accounts,
  onSave,
  onCancel,
}: {
  tx: Transaction;
  accounts: string[];
  onSave: (tx: Transaction) => void;
  onCancel: () => void;
}) {
  const [date, setDate] = useState(tx.date);
  const [narration, setNarration] = useState(tx.narration);
  const [postings, setPostings] = useState<Posting[]>(tx.postings);

  const total = postings.reduce((s, p) => s + (Number(p.amount) || 0), 0);

  return (
    <>
      <tr className="tx-row editing">
        <td>
          <input
            type="date"
            value={date}
            onChange={(e) => setDate(e.target.value)}
          />
        </td>
        <td colSpan={3}>
          <input
            type="text"
            value={narration}
            onChange={(e) => setNarration(e.target.value)}
          />
        </td>
        <td className={`amount ${total === 0 ? "balanced" : "unbalanced"}`}>
          {total.toLocaleString()}
        </td>
        <td className="row-actions">
          <button
            className="btn btn-primary btn-sm"
            onClick={() => onSave({ ...tx, date, narration, postings })}
            disabled={total !== 0}
          >
            保存
          </button>
          <button className="btn btn-secondary btn-sm" onClick={onCancel}>
            取消
          </button>
        </td>
      </tr>
      {postings.map((posting, idx) => (
        <tr key={idx} className="posting-edit-row">
          <td></td>
          <td colSpan={2}>
            <select
              value={posting.account}
              onChange={(e) => {
                const updated = [...postings];
                updated[idx] = { ...updated[idx], account: e.target.value };
                setPostings(updated);
              }}
            >
              {accounts.map((a) => (
                <option key={a} value={a}>
                  {a}
                </option>
              ))}
            </select>
          </td>
          <td>
            <input
              type="number"
              value={posting.amount}
              onChange={(e) => {
                const updated = [...postings];
                updated[idx] = {
                  ...updated[idx],
                  amount: parseInt(e.target.value) || 0,
                };
                setPostings(updated);
              }}
            />
          </td>
          <td></td>
          <td></td>
        </tr>
      ))}
    </>
  );
}
