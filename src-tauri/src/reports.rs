use crate::models::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub date: String,
    pub narration: String,
    pub debit: i64,
    pub credit: i64,
    pub balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLedger {
    pub account: String,
    pub account_type: String,
    pub entries: Vec<LedgerEntry>,
    pub closing_balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralLedger {
    pub accounts: Vec<AccountLedger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitAndLoss {
    pub revenues: Vec<(String, i64)>,
    pub expenses: Vec<(String, i64)>,
    pub total_revenue: i64,
    pub total_expense: i64,
    pub net_income: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub assets: Vec<(String, i64)>,
    pub liabilities: Vec<(String, i64)>,
    pub equity: Vec<(String, i64)>,
    pub total_assets: i64,
    pub total_liabilities: i64,
    pub total_equity: i64,
}

pub fn compute_account_balances(journal: &Journal) -> HashMap<String, i64> {
    let mut balances: HashMap<String, i64> = HashMap::new();

    // Apply opening balances
    for posting in &journal.meta.opening_balances {
        *balances.entry(posting.account.clone()).or_insert(0) += posting.amount;
    }

    // Apply transactions
    for tx in &journal.transactions {
        for posting in &tx.postings {
            *balances.entry(posting.account.clone()).or_insert(0) += posting.amount;
        }
    }

    balances
}

pub fn generate_general_ledger(journal: &Journal) -> GeneralLedger {
    let account_type_map: HashMap<String, &AccountType> = journal
        .accounts
        .iter()
        .map(|a| (a.name.clone(), &a.account_type))
        .collect();

    let mut account_entries: HashMap<String, Vec<LedgerEntry>> = HashMap::new();

    // Initialize accounts
    for account in &journal.accounts {
        account_entries.entry(account.name.clone()).or_default();
    }

    // Apply opening balances as first entry
    let opening_by_account: HashMap<String, i64> = {
        let mut m = HashMap::new();
        for posting in &journal.meta.opening_balances {
            *m.entry(posting.account.clone()).or_insert(0) += posting.amount;
        }
        m
    };

    let mut running_balances: HashMap<String, i64> = opening_by_account.clone();

    // Process transactions sorted by date
    let mut sorted_txs = journal.transactions.clone();
    sorted_txs.sort_by_key(|tx| tx.date);

    for tx in &sorted_txs {
        for posting in &tx.postings {
            let balance = running_balances.entry(posting.account.clone()).or_insert(0);
            let (debit, credit) = if posting.amount >= 0 {
                (posting.amount, 0i64)
            } else {
                (0i64, -posting.amount)
            };
            *balance += posting.amount;
            let entry = LedgerEntry {
                date: tx.date.to_string(),
                narration: tx.narration.clone(),
                debit,
                credit,
                balance: *balance,
            };
            account_entries
                .entry(posting.account.clone())
                .or_default()
                .push(entry);
        }
    }

    let accounts: Vec<AccountLedger> = journal
        .accounts
        .iter()
        .map(|account| {
            let entries = account_entries
                .remove(&account.name)
                .unwrap_or_default();
            let closing_balance = running_balances.get(&account.name).copied().unwrap_or(0);
            let atype_str = match account_type_map.get(&account.name) {
                Some(AccountType::Asset) => "asset",
                Some(AccountType::Liability) => "liability",
                Some(AccountType::Equity) => "equity",
                Some(AccountType::Revenue) => "revenue",
                Some(AccountType::Expense) => "expense",
                None => "unknown",
            };
            AccountLedger {
                account: account.name.clone(),
                account_type: atype_str.to_string(),
                entries,
                closing_balance,
            }
        })
        .collect();

    GeneralLedger { accounts }
}

pub fn generate_profit_and_loss(journal: &Journal) -> ProfitAndLoss {
    let balances = compute_account_balances(journal);

    let mut revenues: Vec<(String, i64)> = vec![];
    let mut expenses: Vec<(String, i64)> = vec![];

    for account in &journal.accounts {
        let balance = balances.get(&account.name).copied().unwrap_or(0);
        match account.account_type {
            AccountType::Revenue => {
                // Revenue accounts: negative balance = credit = revenue
                revenues.push((account.name.clone(), -balance));
            }
            AccountType::Expense => {
                // Expense accounts: positive balance = debit = expense
                expenses.push((account.name.clone(), balance));
            }
            _ => {}
        }
    }

    let total_revenue: i64 = revenues.iter().map(|(_, v)| v).sum();
    let total_expense: i64 = expenses.iter().map(|(_, v)| v).sum();
    let net_income = total_revenue - total_expense;

    ProfitAndLoss {
        revenues,
        expenses,
        total_revenue,
        total_expense,
        net_income,
    }
}

pub fn generate_balance_sheet(journal: &Journal) -> BalanceSheet {
    let balances = compute_account_balances(journal);
    let pl = generate_profit_and_loss(journal);

    let mut assets: Vec<(String, i64)> = vec![];
    let mut liabilities: Vec<(String, i64)> = vec![];
    let mut equity: Vec<(String, i64)> = vec![];

    for account in &journal.accounts {
        let balance = balances.get(&account.name).copied().unwrap_or(0);
        match account.account_type {
            AccountType::Asset => {
                assets.push((account.name.clone(), balance));
            }
            AccountType::Liability => {
                // Liability: negative balance = credit = liability amount
                liabilities.push((account.name.clone(), -balance));
            }
            AccountType::Equity => {
                equity.push((account.name.clone(), -balance));
            }
            _ => {}
        }
    }

    // Add net income to equity
    if pl.net_income != 0 {
        equity.push(("当期純利益".to_string(), pl.net_income));
    }

    let total_assets: i64 = assets.iter().map(|(_, v)| v).sum();
    let total_liabilities: i64 = liabilities.iter().map(|(_, v)| v).sum();
    let total_equity: i64 = equity.iter().map(|(_, v)| v).sum();

    BalanceSheet {
        assets,
        liabilities,
        equity,
        total_assets,
        total_liabilities,
        total_equity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_test_journal() -> Journal {
        let meta = JournalMeta {
            business_name: "テスト".to_string(),
            fiscal_year_start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            fiscal_year_end: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            opening_balances: vec![
                Posting { account: "普通預金".to_string(), amount: 1000000 },
                Posting { account: "元入金".to_string(), amount: -1000000 },
            ],
        };

        let tx1 = Transaction::new(
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            "売上入金".to_string(),
            vec![
                Posting { account: "普通預金".to_string(), amount: 500000 },
                Posting { account: "売上高".to_string(), amount: -500000 },
            ],
        );

        let tx2 = Transaction::new(
            NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            "AWS費用".to_string(),
            vec![
                Posting { account: "通信費".to_string(), amount: 12345 },
                Posting { account: "クレジットカード".to_string(), amount: -12345 },
            ],
        );

        Journal {
            meta,
            accounts: Journal::default_accounts(),
            transactions: vec![tx1, tx2],
        }
    }

    #[test]
    fn test_compute_balances() {
        let journal = make_test_journal();
        let balances = compute_account_balances(&journal);
        assert_eq!(balances.get("普通預金"), Some(&1500000));
        assert_eq!(balances.get("売上高"), Some(&-500000));
        assert_eq!(balances.get("通信費"), Some(&12345));
    }

    #[test]
    fn test_profit_and_loss() {
        let journal = make_test_journal();
        let pl = generate_profit_and_loss(&journal);
        assert_eq!(pl.total_revenue, 500000);
        assert_eq!(pl.total_expense, 12345);
        assert_eq!(pl.net_income, 500000 - 12345);
    }

    #[test]
    fn test_balance_sheet() {
        let journal = make_test_journal();
        let bs = generate_balance_sheet(&journal);
        // Assets = 1500000 (普通預金)
        assert_eq!(bs.total_assets, 1500000);
    }

    #[test]
    fn test_general_ledger() {
        let journal = make_test_journal();
        let gl = generate_general_ledger(&journal);
        let futsuu = gl.accounts.iter().find(|a| a.account == "普通預金").unwrap();
        assert_eq!(futsuu.closing_balance, 1500000);
    }
}
