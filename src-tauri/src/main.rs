#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use aoi_journal_lib::commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_journal,
            new_journal,
            open_journal_from_content,
            save_journal,
            add_transaction,
            update_transaction,
            delete_transaction,
            get_general_ledger,
            get_profit_and_loss,
            get_balance_sheet,
            propose_transaction_from_nlp,
            get_ollama_config,
            set_ollama_config,
            get_accounts,
            add_account,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
