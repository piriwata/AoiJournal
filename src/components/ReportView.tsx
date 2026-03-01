import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface LedgerEntry {
  date: string;
  narration: string;
  debit: number;
  credit: number;
  balance: number;
}

interface AccountLedger {
  account: string;
  account_type: string;
  entries: LedgerEntry[];
  closing_balance: number;
}

interface GeneralLedger {
  accounts: AccountLedger[];
}

interface ProfitAndLoss {
  revenues: [string, number][];
  expenses: [string, number][];
  total_revenue: number;
  total_expense: number;
  net_income: number;
}

interface BalanceSheet {
  assets: [string, number][];
  liabilities: [string, number][];
  equity: [string, number][];
  total_assets: number;
  total_liabilities: number;
  total_equity: number;
}

interface Props {
  reportType: "ledger" | "pl" | "bs";
}

export default function ReportView({ reportType }: Props) {
  const [gl, setGl] = useState<GeneralLedger | null>(null);
  const [pl, setPl] = useState<ProfitAndLoss | null>(null);
  const [bs, setBs] = useState<BalanceSheet | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      setError(null);
      try {
        if (reportType === "ledger") {
          const data = await invoke<GeneralLedger>("get_general_ledger");
          setGl(data);
        } else if (reportType === "pl") {
          const data = await invoke<ProfitAndLoss>("get_profit_and_loss");
          setPl(data);
        } else if (reportType === "bs") {
          const data = await invoke<BalanceSheet>("get_balance_sheet");
          setBs(data);
        }
      } catch (e) {
        setError(String(e));
      }
    };
    load();
  }, [reportType]);

  if (error) return <div className="error-message">{error}</div>;

  if (reportType === "ledger" && gl) {
    const nonEmpty = gl.accounts.filter((a) => a.entries.length > 0 || a.closing_balance !== 0);
    return (
      <div className="report-view">
        <h2>総勘定元帳</h2>
        {nonEmpty.map((account) => (
          <div key={account.account} className="ledger-account">
            <h3>{account.account} <small>({account.account_type})</small></h3>
            <table className="report-table">
              <thead>
                <tr>
                  <th>日付</th>
                  <th>摘要</th>
                  <th>借方</th>
                  <th>貸方</th>
                  <th>残高</th>
                </tr>
              </thead>
              <tbody>
                {account.entries.map((entry, idx) => (
                  <tr key={idx}>
                    <td>{entry.date}</td>
                    <td>{entry.narration}</td>
                    <td className="amount">{entry.debit > 0 ? entry.debit.toLocaleString() : ""}</td>
                    <td className="amount">{entry.credit > 0 ? entry.credit.toLocaleString() : ""}</td>
                    <td className={`amount ${entry.balance < 0 ? "negative" : ""}`}>
                      {Math.abs(entry.balance).toLocaleString()}
                      {entry.balance < 0 ? " (貸)" : ""}
                    </td>
                  </tr>
                ))}
              </tbody>
              <tfoot>
                <tr className="total-row">
                  <td colSpan={4}>期末残高</td>
                  <td className={`amount ${account.closing_balance < 0 ? "negative" : ""}`}>
                    {Math.abs(account.closing_balance).toLocaleString()}
                    {account.closing_balance < 0 ? " (貸)" : ""}
                  </td>
                </tr>
              </tfoot>
            </table>
          </div>
        ))}
      </div>
    );
  }

  if (reportType === "pl" && pl) {
    return (
      <div className="report-view">
        <h2>損益計算書</h2>
        <table className="report-table pl-table">
          <tbody>
            <tr className="section-header-row">
              <td colSpan={2}><strong>収益</strong></td>
            </tr>
            {pl.revenues.filter(([, v]) => v !== 0).map(([name, amount]) => (
              <tr key={name}>
                <td>{name}</td>
                <td className="amount">{amount.toLocaleString()}</td>
              </tr>
            ))}
            <tr className="subtotal-row">
              <td>収益合計</td>
              <td className="amount">{pl.total_revenue.toLocaleString()}</td>
            </tr>
            <tr className="section-header-row">
              <td colSpan={2}><strong>費用</strong></td>
            </tr>
            {pl.expenses.filter(([, v]) => v !== 0).map(([name, amount]) => (
              <tr key={name}>
                <td>{name}</td>
                <td className="amount">{amount.toLocaleString()}</td>
              </tr>
            ))}
            <tr className="subtotal-row">
              <td>費用合計</td>
              <td className="amount">{pl.total_expense.toLocaleString()}</td>
            </tr>
            <tr className={`total-row ${pl.net_income >= 0 ? "profit" : "loss"}`}>
              <td><strong>当期純利益</strong></td>
              <td className="amount"><strong>{pl.net_income.toLocaleString()}</strong></td>
            </tr>
          </tbody>
        </table>
      </div>
    );
  }

  if (reportType === "bs" && bs) {
    return (
      <div className="report-view">
        <h2>貸借対照表</h2>
        <div className="bs-layout">
          <div className="bs-column">
            <h3>資産の部</h3>
            <table className="report-table">
              <tbody>
                {bs.assets.filter(([, v]) => v !== 0).map(([name, amount]) => (
                  <tr key={name}>
                    <td>{name}</td>
                    <td className="amount">{amount.toLocaleString()}</td>
                  </tr>
                ))}
                <tr className="total-row">
                  <td><strong>資産合計</strong></td>
                  <td className="amount"><strong>{bs.total_assets.toLocaleString()}</strong></td>
                </tr>
              </tbody>
            </table>
          </div>
          <div className="bs-column">
            <h3>負債の部</h3>
            <table className="report-table">
              <tbody>
                {bs.liabilities.filter(([, v]) => v !== 0).map(([name, amount]) => (
                  <tr key={name}>
                    <td>{name}</td>
                    <td className="amount">{amount.toLocaleString()}</td>
                  </tr>
                ))}
                <tr className="subtotal-row">
                  <td>負債合計</td>
                  <td className="amount">{bs.total_liabilities.toLocaleString()}</td>
                </tr>
                <h3>純資産の部</h3>
                {bs.equity.filter(([, v]) => v !== 0).map(([name, amount]) => (
                  <tr key={name}>
                    <td>{name}</td>
                    <td className="amount">{amount.toLocaleString()}</td>
                  </tr>
                ))}
                <tr className="subtotal-row">
                  <td>純資産合計</td>
                  <td className="amount">{bs.total_equity.toLocaleString()}</td>
                </tr>
                <tr className="total-row">
                  <td><strong>負債・純資産合計</strong></td>
                  <td className="amount">
                    <strong>{(bs.total_liabilities + bs.total_equity).toLocaleString()}</strong>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    );
  }

  return <div className="loading">読み込み中...</div>;
}
