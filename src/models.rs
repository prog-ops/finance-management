// Mine — Domain models for personal finance

use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Cash,
    Bank,
    EWallet,
    Investment,
    Other,
}

impl AccountType {
    pub const ALL: [AccountType; 5] = [
        Self::Cash,
        Self::Bank,
        Self::EWallet,
        Self::Investment,
        Self::Other,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cash => "Cash",
            Self::Bank => "Bank",
            Self::EWallet => "E-Wallet",
            Self::Investment => "Investment",
            Self::Other => "Other",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "Bank" => Self::Bank,
            "E-Wallet" => Self::EWallet,
            "Investment" => Self::Investment,
            "Other" => Self::Other,
            _ => Self::Cash,
        }
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CategoryKind {
    Income,
    Expense,
}

impl CategoryKind {
    pub const ALL: [CategoryKind; 2] = [Self::Income, Self::Expense];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Income => "Income",
            Self::Expense => "Expense",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "Income" => Self::Income,
            _ => Self::Expense,
        }
    }
}

impl std::fmt::Display for CategoryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TxKind {
    Income,
    Expense,
    Transfer,
}

impl TxKind {
    pub const ALL: [TxKind; 3] = [Self::Income, Self::Expense, Self::Transfer];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Income => "Income",
            Self::Expense => "Expense",
            Self::Transfer => "Transfer",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "Income" => Self::Income,
            "Transfer" => Self::Transfer,
            _ => Self::Expense,
        }
    }
}

impl std::fmt::Display for TxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Frequency {
    pub const ALL: [Frequency; 4] = [Self::Daily, Self::Weekly, Self::Monthly, Self::Yearly];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "Daily",
            Self::Weekly => "Weekly",
            Self::Monthly => "Monthly",
            Self::Yearly => "Yearly",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "Daily" => Self::Daily,
            "Weekly" => Self::Weekly,
            "Yearly" => Self::Yearly,
            _ => Self::Monthly,
        }
    }

    pub fn advance(self, date: NaiveDate) -> NaiveDate {
        match self {
            Self::Daily => date + chrono::Duration::days(1),
            Self::Weekly => date + chrono::Duration::weeks(1),
            Self::Monthly => {
                let (y, m, d) = (date.year(), date.month(), date.day());
                let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                NaiveDate::from_ymd_opt(ny, nm, d.min(28))
                    .or_else(|| NaiveDate::from_ymd_opt(ny, nm, 1))
                    .unwrap_or(date)
            }
            Self::Yearly => {
                let y = date.year() + 1;
                NaiveDate::from_ymd_opt(y, date.month(), date.day())
                    .or_else(|| NaiveDate::from_ymd_opt(y, date.month(), 28))
                    .unwrap_or(date)
            }
        }
    }
}

impl std::fmt::Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub account_type: AccountType,
    pub opening_balance: i64,
    pub notes: String,
    pub balance: i64,
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Account {}

impl std::fmt::Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, format_balance_short(self.balance))
    }
}

fn format_balance_short(amount: i64) -> String {
    crate::util::format_idr(amount)
}

#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub kind: CategoryKind,
}

impl PartialEq for Category {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Category {}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{}]", self.name, self.kind)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Transaction {
    pub id: i64,
    pub account_id: i64,
    pub category_id: i64,
    pub amount: i64,
    pub kind: TxKind,
    pub to_account_id: Option<i64>,
    pub date: NaiveDate,
    pub note: String,
    pub account_name: String,
    pub category_name: String,
    pub to_account_name: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Budget {
    pub id: i64,
    pub category_id: i64,
    pub year_month: String,
    pub limit_amount: i64,
    pub category_name: String,
    pub spent: i64,
}

#[derive(Debug, Clone)]
pub struct Recurring {
    pub id: i64,
    pub account_id: i64,
    pub category_id: i64,
    pub amount: i64,
    pub kind: TxKind,
    pub frequency: Frequency,
    pub next_date: NaiveDate,
    pub note: String,
    pub active: bool,
    pub account_name: String,
    pub category_name: String,
}

#[derive(Debug, Clone, Default)]
pub struct MonthSummary {
    pub income: i64,
    pub expense: i64,
}

impl MonthSummary {
    pub fn net(&self) -> i64 {
        self.income - self.expense
    }
}

#[derive(Debug, Clone)]
pub struct CategorySpend {
    pub name: String,
    pub amount: i64,
}
