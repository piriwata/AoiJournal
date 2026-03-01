use crate::models::*;
use crate::ollama::{OllamaConfig, ProposedTransaction};
use crate::parser::{parse_journal, serialize_journal};
use crate::reports::{generate_balance_sheet, generate_general_ledger, generate_profit_and_loss};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub journal: Mutex<Option<Journal>>,
    pub file_path: Mutex<Option<String>>,
    pub ollama_config: Mutex<OllamaConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            journal: Mutex::new(None),
            file_path: Mutex::new(None),
            ollama_config: Mutex::new(OllamaConfig::default()),
        }
    }
}

#[tauri::command]
pub fn get_journal(state: State<AppState>) -> Option<Journal> {
    state.journal.lock().unwrap().clone()
}

#[tauri::command]
pub fn new_journal(
    state: State<AppState>,
    business_name: String,
    fiscal_year_start: String,
    fiscal_year_end: String,
) -> Result<Journal, String> {
    use chrono::NaiveDate;
    let start = NaiveDate::parse_from_str(&fiscal_year_start, "%Y-%m-%d")
        .map_err(|e| e.to_string())?;
    let end = NaiveDate::parse_from_str(&fiscal_year_end, "%Y-%m-%d")
        .map_err(|e| e.to_string())?;

    let meta = JournalMeta {
        business_name,
        fiscal_year_start: start,
        fiscal_year_end: end,
        opening_balances: vec![],
    };

    let journal = Journal {
        meta,
        accounts: Journal::default_accounts(),
        transactions: vec![],
    };

    *state.journal.lock().unwrap() = Some(journal.clone());
    Ok(journal)
}

#[tauri::command]
pub fn open_journal_from_content(
    state: State<AppState>,
    content: String,
    path: String,
) -> Result<Journal, String> {
    let journal = parse_journal(&content)?;
    *state.file_path.lock().unwrap() = Some(path);
    *state.journal.lock().unwrap() = Some(journal.clone());
    Ok(journal)
}

#[tauri::command]
pub fn save_journal(state: State<AppState>) -> Result<String, String> {
    let journal = state.journal.lock().unwrap();
    if let Some(ref j) = *journal {
        Ok(serialize_journal(j))
    } else {
        Err("No journal loaded".to_string())
    }
}

#[tauri::command]
pub fn add_transaction(
    state: State<AppState>,
    date: String,
    narration: String,
    postings: Vec<Posting>,
    memo: Option<String>,
) -> Result<Transaction, String> {
    use chrono::NaiveDate;
    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| e.to_string())?;
    let mut tx = Transaction::new(date, narration, postings);
    tx.memo = memo;

    if !tx.is_balanced() {
        return Err("仕訳の借方・貸方が一致しません".to_string());
    }

    let mut journal = state.journal.lock().unwrap();
    if let Some(ref mut j) = *journal {
        j.transactions.push(tx.clone());
        Ok(tx)
    } else {
        Err("No journal loaded".to_string())
    }
}

#[tauri::command]
pub fn update_transaction(
    state: State<AppState>,
    id: String,
    date: String,
    narration: String,
    postings: Vec<Posting>,
    memo: Option<String>,
) -> Result<Transaction, String> {
    use chrono::NaiveDate;
    let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| e.to_string())?;
    let mut tx = Transaction::new(date, narration, postings);
    tx.id = id.clone();
    tx.memo = memo;

    if !tx.is_balanced() {
        return Err("仕訳の借方・貸方が一致しません".to_string());
    }

    let mut journal = state.journal.lock().unwrap();
    if let Some(ref mut j) = *journal {
        if let Some(existing) = j.transactions.iter_mut().find(|t| t.id == id) {
            *existing = tx.clone();
            Ok(tx)
        } else {
            Err(format!("Transaction {} not found", id))
        }
    } else {
        Err("No journal loaded".to_string())
    }
}

#[tauri::command]
pub fn delete_transaction(state: State<AppState>, id: String) -> Result<(), String> {
    let mut journal = state.journal.lock().unwrap();
    if let Some(ref mut j) = *journal {
        let before = j.transactions.len();
        j.transactions.retain(|t| t.id != id);
        if j.transactions.len() == before {
            Err(format!("Transaction {} not found", id))
        } else {
            Ok(())
        }
    } else {
        Err("No journal loaded".to_string())
    }
}

#[tauri::command]
pub fn get_general_ledger(
    state: State<AppState>,
) -> Result<crate::reports::GeneralLedger, String> {
    let journal = state.journal.lock().unwrap();
    if let Some(ref j) = *journal {
        Ok(generate_general_ledger(j))
    } else {
        Err("No journal loaded".to_string())
    }
}

#[tauri::command]
pub fn get_profit_and_loss(
    state: State<AppState>,
) -> Result<crate::reports::ProfitAndLoss, String> {
    let journal = state.journal.lock().unwrap();
    if let Some(ref j) = *journal {
        Ok(generate_profit_and_loss(j))
    } else {
        Err("No journal loaded".to_string())
    }
}

#[tauri::command]
pub fn get_balance_sheet(
    state: State<AppState>,
) -> Result<crate::reports::BalanceSheet, String> {
    let journal = state.journal.lock().unwrap();
    if let Some(ref j) = *journal {
        Ok(generate_balance_sheet(j))
    } else {
        Err("No journal loaded".to_string())
    }
}

#[tauri::command]
pub async fn propose_transaction_from_nlp(
    state: State<'_, AppState>,
    text: String,
) -> Result<Vec<ProposedTransaction>, String> {
    let config = state.ollama_config.lock().unwrap().clone();
    let journal = state.journal.lock().unwrap().clone();
    let accounts: Vec<String> = if let Some(j) = journal {
        j.accounts.iter().map(|a| a.name.clone()).collect()
    } else {
        Journal::default_accounts()
            .iter()
            .map(|a| a.name.clone())
            .collect()
    };
    crate::ollama::propose_transaction(&config, &text, &accounts).await
}

#[tauri::command]
pub fn get_ollama_config(state: State<AppState>) -> OllamaConfig {
    state.ollama_config.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_ollama_config(
    state: State<AppState>,
    base_url: String,
    model: String,
    timeout_secs: u64,
) -> Result<(), String> {
    let mut config = state.ollama_config.lock().unwrap();
    config.base_url = base_url;
    config.model = model;
    config.timeout_secs = timeout_secs;
    Ok(())
}

#[tauri::command]
pub fn get_accounts(state: State<AppState>) -> Vec<Account> {
    let journal = state.journal.lock().unwrap();
    if let Some(ref j) = *journal {
        j.accounts.clone()
    } else {
        Journal::default_accounts()
    }
}

#[tauri::command]
pub fn add_account(
    state: State<AppState>,
    name: String,
    account_type: AccountType,
) -> Result<(), String> {
    let mut journal = state.journal.lock().unwrap();
    if let Some(ref mut j) = *journal {
        if j.accounts.iter().any(|a| a.name == name) {
            return Err(format!("Account {} already exists", name));
        }
        j.accounts.push(Account { name, account_type });
        Ok(())
    } else {
        Err("No journal loaded".to_string())
    }
}
