use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountType {
    Asset,      // 資産
    Liability,  // 負債
    Equity,     // 純資産・資本
    Revenue,    // 収益
    Expense,    // 費用
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub name: String,
    pub account_type: AccountType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Posting {
    pub account: String,
    pub amount: i64, // positive = debit, negative = credit (JPY)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub date: NaiveDate,
    pub narration: String,
    pub postings: Vec<Posting>,
    pub memo: Option<String>,
}

impl Transaction {
    pub fn new(date: NaiveDate, narration: String, postings: Vec<Posting>) -> Self {
        Transaction {
            id: Uuid::new_v4().to_string(),
            date,
            narration,
            postings,
            memo: None,
        }
    }

    pub fn is_balanced(&self) -> bool {
        let sum: i64 = self.postings.iter().map(|p| p.amount).sum();
        sum == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalMeta {
    pub business_name: String,
    pub fiscal_year_start: NaiveDate,
    pub fiscal_year_end: NaiveDate,
    pub opening_balances: Vec<Posting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub meta: JournalMeta,
    pub accounts: Vec<Account>,
    pub transactions: Vec<Transaction>,
}

impl Journal {
    pub fn default_accounts() -> Vec<Account> {
        vec![
            // Assets 資産
            Account { name: "現金".to_string(), account_type: AccountType::Asset },
            Account { name: "普通預金".to_string(), account_type: AccountType::Asset },
            Account { name: "売掛金".to_string(), account_type: AccountType::Asset },
            Account { name: "前払費用".to_string(), account_type: AccountType::Asset },
            // Liabilities 負債
            Account { name: "未払金".to_string(), account_type: AccountType::Liability },
            Account { name: "借入金".to_string(), account_type: AccountType::Liability },
            Account { name: "前受金".to_string(), account_type: AccountType::Liability },
            Account { name: "クレジットカード".to_string(), account_type: AccountType::Liability },
            // Equity 純資産
            Account { name: "元入金".to_string(), account_type: AccountType::Equity },
            Account { name: "事業主貸".to_string(), account_type: AccountType::Equity },
            Account { name: "事業主借".to_string(), account_type: AccountType::Equity },
            // Revenue 収益
            Account { name: "売上高".to_string(), account_type: AccountType::Revenue },
            Account { name: "雑収入".to_string(), account_type: AccountType::Revenue },
            // Expenses 費用
            Account { name: "通信費".to_string(), account_type: AccountType::Expense },
            Account { name: "旅費交通費".to_string(), account_type: AccountType::Expense },
            Account { name: "消耗品費".to_string(), account_type: AccountType::Expense },
            Account { name: "新聞図書費".to_string(), account_type: AccountType::Expense },
            Account { name: "接待交際費".to_string(), account_type: AccountType::Expense },
            Account { name: "地代家賃".to_string(), account_type: AccountType::Expense },
            Account { name: "水道光熱費".to_string(), account_type: AccountType::Expense },
            Account { name: "支払手数料".to_string(), account_type: AccountType::Expense },
            Account { name: "雑費".to_string(), account_type: AccountType::Expense },
            Account { name: "減価償却費".to_string(), account_type: AccountType::Expense },
            Account { name: "給料賃金".to_string(), account_type: AccountType::Expense },
            Account { name: "福利厚生費".to_string(), account_type: AccountType::Expense },
        ]
    }
}
