// Mine — SQLite persistence layer

use anyhow::{Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use directories::ProjectDirs;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;

use crate::models::*;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open_default() -> Result<Self> {
        let path = db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open database at {}", path.display()))?;
        let db = Self { conn };
        db.migrate()?;
        db.seed_if_empty()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                account_type TEXT NOT NULL,
                opening_balance INTEGER NOT NULL DEFAULT 0,
                notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                kind TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                amount INTEGER NOT NULL CHECK(amount >= 0),
                kind TEXT NOT NULL,
                to_account_id INTEGER,
                date TEXT NOT NULL,
                note TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE,
                FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE RESTRICT,
                FOREIGN KEY(to_account_id) REFERENCES accounts(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS budgets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category_id INTEGER NOT NULL,
                year_month TEXT NOT NULL,
                limit_amount INTEGER NOT NULL CHECK(limit_amount >= 0),
                UNIQUE(category_id, year_month),
                FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS recurring (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account_id INTEGER NOT NULL,
                category_id INTEGER NOT NULL,
                amount INTEGER NOT NULL CHECK(amount >= 0),
                kind TEXT NOT NULL,
                frequency TEXT NOT NULL,
                next_date TEXT NOT NULL,
                note TEXT NOT NULL DEFAULT '',
                active INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE,
                FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE RESTRICT
            );

            CREATE INDEX IF NOT EXISTS idx_tx_date ON transactions(date);
            CREATE INDEX IF NOT EXISTS idx_tx_account ON transactions(account_id);
            CREATE INDEX IF NOT EXISTS idx_tx_category ON transactions(category_id);
            "#,
        )?;
        Ok(())
    }

    fn seed_if_empty(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))?;
        if count > 0 {
            return Ok(());
        }

        let now = Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO accounts (name, account_type, opening_balance, notes, created_at)
             VALUES (?1, ?2, 0, '', ?3)",
            params!["Cash", "Cash", now],
        )?;

        let defaults = [
            ("Salary", "Income"),
            ("Freelance", "Income"),
            ("Gift", "Income"),
            ("Other Income", "Income"),
            ("Food", "Expense"),
            ("Transport", "Expense"),
            ("Housing", "Expense"),
            ("Utilities", "Expense"),
            ("Shopping", "Expense"),
            ("Health", "Expense"),
            ("Entertainment", "Expense"),
            ("Education", "Expense"),
            ("Transfer", "Expense"),
            ("Other Expense", "Expense"),
        ];

        for (name, kind) in defaults {
            self.conn.execute(
                "INSERT INTO categories (name, kind, created_at) VALUES (?1, ?2, ?3)",
                params![name, kind, now],
            )?;
        }
        Ok(())
    }

    // —— Accounts ——

    pub fn list_accounts(&self) -> Result<Vec<Account>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.name, a.account_type, a.opening_balance, a.notes,
                a.opening_balance
                  + COALESCE((
                      SELECT SUM(CASE
                        WHEN t.kind = 'Income' THEN t.amount
                        WHEN t.kind = 'Expense' THEN -t.amount
                        WHEN t.kind = 'Transfer' THEN -t.amount
                        ELSE 0 END)
                      FROM transactions t WHERE t.account_id = a.id
                  ), 0)
                  + COALESCE((
                      SELECT SUM(t.amount) FROM transactions t
                      WHERE t.kind = 'Transfer' AND t.to_account_id = a.id
                  ), 0) AS balance
             FROM accounts a
             ORDER BY a.name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Account {
                id: row.get(0)?,
                name: row.get(1)?,
                account_type: AccountType::parse(&row.get::<_, String>(2)?),
                opening_balance: row.get(3)?,
                notes: row.get(4)?,
                balance: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn add_account(
        &self,
        name: &str,
        account_type: AccountType,
        opening_balance: i64,
        notes: &str,
    ) -> Result<()> {
        let now = Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO accounts (name, account_type, opening_balance, notes, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, account_type.as_str(), opening_balance, notes, now],
        )?;
        Ok(())
    }

    pub fn update_account(
        &self,
        id: i64,
        name: &str,
        account_type: AccountType,
        opening_balance: i64,
        notes: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE accounts SET name=?1, account_type=?2, opening_balance=?3, notes=?4 WHERE id=?5",
            params![name, account_type.as_str(), opening_balance, notes, id],
        )?;
        Ok(())
    }

    pub fn delete_account(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM accounts WHERE id=?1", params![id])?;
        Ok(())
    }

    // —— Categories ——

    pub fn list_categories(&self) -> Result<Vec<Category>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, kind FROM categories ORDER BY kind, name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: CategoryKind::parse(&row.get::<_, String>(2)?),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn add_category(&self, name: &str, kind: CategoryKind) -> Result<()> {
        let now = Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO categories (name, kind, created_at) VALUES (?1, ?2, ?3)",
            params![name, kind.as_str(), now],
        )?;
        Ok(())
    }

    pub fn update_category(&self, id: i64, name: &str, kind: CategoryKind) -> Result<()> {
        self.conn.execute(
            "UPDATE categories SET name=?1, kind=?2 WHERE id=?3",
            params![name, kind.as_str(), id],
        )?;
        Ok(())
    }

    pub fn delete_category(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM categories WHERE id=?1", params![id])?;
        Ok(())
    }

    // —— Transactions ——

    pub fn list_transactions(&self, limit: i64) -> Result<Vec<Transaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.account_id, t.category_id, t.amount, t.kind, t.to_account_id,
                    t.date, t.note, a.name, c.name,
                    (SELECT name FROM accounts WHERE id = t.to_account_id)
             FROM transactions t
             JOIN accounts a ON a.id = t.account_id
             JOIN categories c ON c.id = t.category_id
             ORDER BY t.date DESC, t.id DESC
             LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], map_transaction)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn add_transaction(
        &self,
        account_id: i64,
        category_id: i64,
        amount: i64,
        kind: TxKind,
        to_account_id: Option<i64>,
        date: NaiveDate,
        note: &str,
    ) -> Result<()> {
        let now = Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO transactions
             (account_id, category_id, amount, kind, to_account_id, date, note, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                account_id,
                category_id,
                amount,
                kind.as_str(),
                to_account_id,
                date.to_string(),
                note,
                now
            ],
        )?;
        Ok(())
    }

    pub fn delete_transaction(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM transactions WHERE id=?1", params![id])?;
        Ok(())
    }

    // —— Budgets ——

    pub fn list_budgets(&self, year_month: &str) -> Result<Vec<Budget>> {
        let mut stmt = self.conn.prepare(
            "SELECT b.id, b.category_id, b.year_month, b.limit_amount, c.name,
                COALESCE((
                  SELECT SUM(t.amount) FROM transactions t
                  WHERE t.category_id = b.category_id
                    AND t.kind = 'Expense'
                    AND substr(t.date, 1, 7) = b.year_month
                ), 0) AS spent
             FROM budgets b
             JOIN categories c ON c.id = b.category_id
             WHERE b.year_month = ?1
             ORDER BY c.name",
        )?;
        let rows = stmt.query_map(params![year_month], |row| {
            Ok(Budget {
                id: row.get(0)?,
                category_id: row.get(1)?,
                year_month: row.get(2)?,
                limit_amount: row.get(3)?,
                category_name: row.get(4)?,
                spent: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn upsert_budget(&self, category_id: i64, year_month: &str, limit_amount: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO budgets (category_id, year_month, limit_amount)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(category_id, year_month)
             DO UPDATE SET limit_amount = excluded.limit_amount",
            params![category_id, year_month, limit_amount],
        )?;
        Ok(())
    }

    pub fn delete_budget(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM budgets WHERE id=?1", params![id])?;
        Ok(())
    }

    // —— Recurring ——

    pub fn list_recurring(&self) -> Result<Vec<Recurring>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.id, r.account_id, r.category_id, r.amount, r.kind, r.frequency,
                    r.next_date, r.note, r.active, a.name, c.name
             FROM recurring r
             JOIN accounts a ON a.id = r.account_id
             JOIN categories c ON c.id = r.category_id
             ORDER BY r.active DESC, r.next_date",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Recurring {
                id: row.get(0)?,
                account_id: row.get(1)?,
                category_id: row.get(2)?,
                amount: row.get(3)?,
                kind: TxKind::parse(&row.get::<_, String>(4)?),
                frequency: Frequency::parse(&row.get::<_, String>(5)?),
                next_date: NaiveDate::parse_from_str(&row.get::<_, String>(6)?, "%Y-%m-%d")
                    .unwrap_or_else(|_| Local::now().date_naive()),
                note: row.get(7)?,
                active: row.get::<_, i64>(8)? != 0,
                account_name: row.get(9)?,
                category_name: row.get(10)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn add_recurring(
        &self,
        account_id: i64,
        category_id: i64,
        amount: i64,
        kind: TxKind,
        frequency: Frequency,
        next_date: NaiveDate,
        note: &str,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO recurring
             (account_id, category_id, amount, kind, frequency, next_date, note, active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
            params![
                account_id,
                category_id,
                amount,
                kind.as_str(),
                frequency.as_str(),
                next_date.to_string(),
                note
            ],
        )?;
        Ok(())
    }

    pub fn set_recurring_active(&self, id: i64, active: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE recurring SET active=?1 WHERE id=?2",
            params![if active { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    pub fn delete_recurring(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM recurring WHERE id=?1", params![id])?;
        Ok(())
    }

    /// Apply due recurring rules into real transactions.
    pub fn process_due_recurring(&self) -> Result<usize> {
        let today = Local::now().date_naive();
        let items = self.list_recurring()?;
        let mut count = 0usize;

        for item in items {
            if !item.active {
                continue;
            }
            let mut next = item.next_date;
            while next <= today {
                self.add_transaction(
                    item.account_id,
                    item.category_id,
                    item.amount,
                    item.kind,
                    None,
                    next,
                    &format!("Recurring: {}", item.note),
                )?;
                next = item.frequency.advance(next);
                count += 1;
            }
            self.conn.execute(
                "UPDATE recurring SET next_date=?1 WHERE id=?2",
                params![next.to_string(), item.id],
            )?;
        }
        Ok(count)
    }

    // —— Reports ——

    pub fn month_summary(&self, year_month: &str) -> Result<MonthSummary> {
        let income: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount),0) FROM transactions
                 WHERE kind='Income' AND substr(date,1,7)=?1",
                params![year_month],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0);
        let expense: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(amount),0) FROM transactions
                 WHERE kind='Expense' AND substr(date,1,7)=?1",
                params![year_month],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0);
        Ok(MonthSummary { income, expense })
    }

    pub fn expense_by_category(&self, year_month: &str) -> Result<Vec<CategorySpend>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.name, COALESCE(SUM(t.amount),0)
             FROM transactions t
             JOIN categories c ON c.id = t.category_id
             WHERE t.kind='Expense' AND substr(t.date,1,7)=?1
             GROUP BY c.id
             ORDER BY 2 DESC",
        )?;
        let rows = stmt.query_map(params![year_month], |row| {
            Ok(CategorySpend {
                name: row.get(0)?,
                amount: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn monthly_totals(&self, months: usize) -> Result<Vec<(String, MonthSummary)>> {
        let today = Local::now().date_naive();
        let mut out = Vec::new();
        for i in (0..months).rev() {
            let d = today
                .checked_sub_signed(chrono::Duration::days(30 * i as i64))
                .unwrap_or(today);
            let ym = format!("{:04}-{:02}", d.year(), d.month());
            // Prefer calendar months walking back
            let _ = ym;
        }

        // Walk calendar months properly
        let mut y = today.year();
        let mut m = today.month();
        let mut keys = Vec::new();
        for _ in 0..months {
            keys.push(format!("{:04}-{:02}", y, m));
            if m == 1 {
                m = 12;
                y -= 1;
            } else {
                m -= 1;
            }
        }
        keys.reverse();

        for ym in keys {
            let summary = self.month_summary(&ym)?;
            out.push((ym, summary));
        }
        Ok(out)
    }

    pub fn total_balance(&self) -> Result<i64> {
        Ok(self.list_accounts()?.iter().map(|a| a.balance).sum())
    }

    pub fn export_transactions_csv(&self, path: &std::path::Path) -> Result<usize> {
        let txs = self.list_transactions(100_000)?;
        let mut wtr = csv::Writer::from_path(path)?;
        wtr.write_record([
            "id",
            "date",
            "kind",
            "account",
            "category",
            "to_account",
            "amount_idr",
            "note",
        ])?;
        for t in &txs {
            wtr.write_record(&[
                t.id.to_string(),
                t.date.to_string(),
                t.kind.as_str().to_string(),
                t.account_name.clone(),
                t.category_name.clone(),
                t.to_account_name.clone().unwrap_or_default(),
                t.amount.to_string(),
                t.note.clone(),
            ])?;
        }
        wtr.flush()?;
        Ok(txs.len())
    }
}

fn map_transaction(row: &rusqlite::Row<'_>) -> rusqlite::Result<Transaction> {
    let date_str: String = row.get(6)?;
    Ok(Transaction {
        id: row.get(0)?,
        account_id: row.get(1)?,
        category_id: row.get(2)?,
        amount: row.get(3)?,
        kind: TxKind::parse(&row.get::<_, String>(4)?),
        to_account_id: row.get(5)?,
        date: NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").unwrap_or_else(|_| {
            Local::now().date_naive()
        }),
        note: row.get(7)?,
        account_name: row.get(8)?,
        category_name: row.get(9)?,
        to_account_name: row.get(10)?,
    })
}

fn db_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "FinanceManager", "FinanceManager")
        .context("Could not resolve application data directory")?;
    Ok(dirs.data_dir().join("finance.db"))
}

