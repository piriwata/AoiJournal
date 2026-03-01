use crate::models::*;
use chrono::NaiveDate;
use std::collections::HashMap;

pub fn serialize_journal(journal: &Journal) -> String {
    let mut output = String::new();

    // Write metadata header
    output.push_str("; === Aoi Journal ===\n");
    output.push_str(&format!("; business: {}\n", journal.meta.business_name));
    output.push_str(&format!(
        "; fiscal_year: {} - {}\n",
        journal.meta.fiscal_year_start, journal.meta.fiscal_year_end
    ));
    output.push('\n');

    // Write account declarations
    for account in &journal.accounts {
        let atype = match account.account_type {
            AccountType::Asset => "asset",
            AccountType::Liability => "liability",
            AccountType::Equity => "equity",
            AccountType::Revenue => "revenue",
            AccountType::Expense => "expense",
        };
        output.push_str(&format!("account {} type:{}\n", account.name, atype));
    }
    output.push('\n');

    // Write opening balances if any
    if !journal.meta.opening_balances.is_empty() {
        let fy = journal.meta.fiscal_year_start;
        output.push_str(&format!("{} 期首残高\n", fy));
        for posting in &journal.meta.opening_balances {
            output.push_str(&format!("    {}  {} JPY\n", posting.account, posting.amount));
        }
        output.push('\n');
    }

    // Write transactions
    for tx in &journal.transactions {
        if let Some(memo) = &tx.memo {
            output.push_str(&format!("; {}\n", memo));
        }
        output.push_str(&format!("; id:{}\n", tx.id));
        output.push_str(&format!("{} {}\n", tx.date, tx.narration));
        for posting in &tx.postings {
            output.push_str(&format!("    {}  {} JPY\n", posting.account, posting.amount));
        }
        output.push('\n');
    }

    output
}

pub fn parse_journal(content: &str) -> Result<Journal, String> {
    let mut meta = JournalMeta {
        business_name: "".to_string(),
        fiscal_year_start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        fiscal_year_end: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        opening_balances: vec![],
    };
    let mut accounts: Vec<Account> = vec![];
    let mut transactions: Vec<Transaction> = vec![];
    let mut account_map: HashMap<String, AccountType> = HashMap::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut pending_id: Option<String> = None;
    let mut pending_memo: Option<String> = None;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        // Comment lines
        if trimmed.starts_with(';') {
            let comment = trimmed.trim_start_matches(';').trim();
            if let Some(val) = comment.strip_prefix("business:") {
                meta.business_name = val.trim().to_string();
            } else if let Some(val) = comment.strip_prefix("fiscal_year:") {
                let parts: Vec<&str> = val.trim().split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(start), Ok(end)) = (
                        NaiveDate::parse_from_str(parts[0].trim(), "%Y-%m-%d"),
                        NaiveDate::parse_from_str(parts[1].trim(), "%Y-%m-%d"),
                    ) {
                        meta.fiscal_year_start = start;
                        meta.fiscal_year_end = end;
                    }
                }
            } else if let Some(val) = comment.strip_prefix("id:") {
                pending_id = Some(val.trim().to_string());
            } else {
                pending_memo = Some(comment.to_string());
            }
            i += 1;
            continue;
        }

        // Account declaration
        if trimmed.starts_with("account ") {
            let rest = trimmed.trim_start_matches("account ").trim();
            let parts: Vec<&str> = rest.splitn(2, " type:").collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let atype = match parts[1].trim() {
                    "asset" => AccountType::Asset,
                    "liability" => AccountType::Liability,
                    "equity" => AccountType::Equity,
                    "revenue" => AccountType::Revenue,
                    "expense" => AccountType::Expense,
                    _ => AccountType::Expense,
                };
                account_map.insert(name.clone(), atype.clone());
                accounts.push(Account { name, account_type: atype });
            }
            i += 1;
            continue;
        }

        // Transaction header: starts with date YYYY-MM-DD
        if let Some(date) = parse_date_prefix(trimmed) {
            let narration = trimmed[10..].trim().to_string();
            let mut postings: Vec<Posting> = vec![];
            i += 1;

            while i < lines.len() {
                let pline = lines[i];
                let ptrimmed = pline.trim();

                if ptrimmed.is_empty() || (!pline.starts_with(' ') && !pline.starts_with('\t')) {
                    break;
                }

                if let Some(posting) = parse_posting_line(ptrimmed) {
                    postings.push(posting);
                }
                i += 1;
            }

            let tx_id = pending_id.take().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            let memo = pending_memo.take();

            // Check if this is opening balance (期首残高)
            if narration.contains("期首残高") {
                meta.opening_balances = postings;
            } else {
                let mut tx = Transaction::new(date, narration, postings);
                tx.id = tx_id;
                tx.memo = memo;
                transactions.push(tx);
            }
            continue;
        }

        i += 1;
    }

    // If no accounts defined, use defaults
    if accounts.is_empty() {
        accounts = Journal::default_accounts();
    }

    // Suppress unused variable warning
    let _ = account_map;

    Ok(Journal {
        meta,
        accounts,
        transactions,
    })
}

fn parse_date_prefix(s: &str) -> Option<NaiveDate> {
    if s.len() < 10 {
        return None;
    }
    NaiveDate::parse_from_str(&s[..10], "%Y-%m-%d").ok()
}

fn parse_posting_line(s: &str) -> Option<Posting> {
    // Format: "account_name  amount JPY"
    // Find the amount by splitting on multiple spaces or tab
    let s = s.trim();
    // Try to find "  " (two or more spaces) separator
    let sep_pos = s.find("  ")?;
    let account = s[..sep_pos].trim().to_string();
    let rest = s[sep_pos..].trim();
    // rest is like "12345 JPY" or "-12345 JPY"
    let amount_str = rest.split_whitespace().next()?;
    let amount: i64 = amount_str.parse().ok()?;
    Some(Posting { account, amount })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_transaction() {
        let content = r#"
; business: テスト株式会社
; fiscal_year: 2024-01-01 - 2024-12-31

account 現金 type:asset
account 売上高 type:revenue

2024-03-01 売上入金
    現金  100000 JPY
    売上高  -100000 JPY
"#;
        let journal = parse_journal(content).unwrap();
        assert_eq!(journal.transactions.len(), 1);
        let tx = &journal.transactions[0];
        assert_eq!(tx.narration, "売上入金");
        assert_eq!(tx.postings.len(), 2);
        assert!(tx.is_balanced());
    }

    #[test]
    fn test_serialize_roundtrip() {
        let meta = JournalMeta {
            business_name: "テスト".to_string(),
            fiscal_year_start: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            fiscal_year_end: NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            opening_balances: vec![],
        };
        let mut journal = Journal {
            meta,
            accounts: Journal::default_accounts(),
            transactions: vec![],
        };
        let tx = Transaction::new(
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
            "AWS費用".to_string(),
            vec![
                Posting { account: "通信費".to_string(), amount: 12345 },
                Posting { account: "クレジットカード".to_string(), amount: -12345 },
            ],
        );
        journal.transactions.push(tx);
        let serialized = serialize_journal(&journal);
        let parsed = parse_journal(&serialized).unwrap();
        assert_eq!(parsed.transactions.len(), 1);
        assert!(parsed.transactions[0].is_balanced());
    }

    #[test]
    fn test_unbalanced_transaction() {
        let tx = Transaction::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            "不均衡".to_string(),
            vec![
                Posting { account: "現金".to_string(), amount: 1000 },
                Posting { account: "売上高".to_string(), amount: -999 },
            ],
        );
        assert!(!tx.is_balanced());
    }

    #[test]
    fn test_opening_balances() {
        let content = r#"
; business: テスト
; fiscal_year: 2024-01-01 - 2024-12-31

account 現金 type:asset
account 元入金 type:equity

2024-01-01 期首残高
    現金  500000 JPY
    元入金  -500000 JPY
"#;
        let journal = parse_journal(content).unwrap();
        assert_eq!(journal.meta.opening_balances.len(), 2);
        assert_eq!(journal.transactions.len(), 0);
    }
}
