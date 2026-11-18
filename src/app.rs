// Mine — Application state, update, and Windows XP views

use chrono::{Local, NaiveDate};
use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Fill, Task};

use crate::chart::BarChart;
use crate::db::Database;
use crate::export;
use crate::models::*;
use crate::theme::{
    face_container, inset_panel, label_text, section_header, title_bar, title_text, white_panel,
    xp_button, xp_input, xp_tab, XP_DANGER, XP_SUCCESS, XP_TEXT,
};
use crate::util::{current_year_month, format_idr, parse_amount};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Transactions,
    Accounts,
    Categories,
    Budgets,
    Recurring,
    Reports,
    Export,
}

impl Tab {
    const ALL: [Tab; 8] = [
        Self::Dashboard,
        Self::Transactions,
        Self::Accounts,
        Self::Categories,
        Self::Budgets,
        Self::Recurring,
        Self::Reports,
        Self::Export,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Transactions => "Transactions",
            Self::Accounts => "Accounts",
            Self::Categories => "Categories",
            Self::Budgets => "Budgets",
            Self::Recurring => "Recurring",
            Self::Reports => "Reports",
            Self::Export => "Export",
        }
    }
}

pub struct App {
    db: Database,
    tab: Tab,
    status: String,

    accounts: Vec<Account>,
    categories: Vec<Category>,
    transactions: Vec<Transaction>,
    budgets: Vec<Budget>,
    recurring: Vec<Recurring>,
    summary: MonthSummary,
    by_category: Vec<CategorySpend>,
    monthly: Vec<(String, MonthSummary)>,
    total_balance: i64,

    // forms
    tx_amount: String,
    tx_note: String,
    tx_date: String,
    tx_kind: TxKind,
    tx_account: Option<Account>,
    tx_category: Option<Category>,
    tx_to_account: Option<Account>,

    acc_name: String,
    acc_notes: String,
    acc_opening: String,
    acc_type: AccountType,
    editing_account: Option<i64>,

    cat_name: String,
    cat_kind: CategoryKind,
    editing_category: Option<i64>,

    budget_amount: String,
    budget_category: Option<Category>,
    budget_month: String,

    rec_amount: String,
    rec_note: String,
    rec_date: String,
    rec_kind: TxKind,
    rec_freq: Frequency,
    rec_account: Option<Account>,
    rec_category: Option<Category>,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(Tab),
    Refresh,

    // transactions
    TxAmountChanged(String),
    TxNoteChanged(String),
    TxDateChanged(String),
    TxKindSelected(TxKind),
    TxAccountSelected(Account),
    TxCategorySelected(Category),
    TxToAccountSelected(Account),
    AddTransaction,
    DeleteTransaction(i64),

    // accounts
    AccNameChanged(String),
    AccNotesChanged(String),
    AccOpeningChanged(String),
    AccTypeSelected(AccountType),
    SaveAccount,
    EditAccount(i64),
    DeleteAccount(i64),
    CancelEditAccount,

    // categories
    CatNameChanged(String),
    CatKindSelected(CategoryKind),
    SaveCategory,
    EditCategory(i64),
    DeleteCategory(i64),
    CancelEditCategory,

    // budgets
    BudgetAmountChanged(String),
    BudgetCategorySelected(Category),
    BudgetMonthChanged(String),
    SaveBudget,
    DeleteBudget(i64),

    // recurring
    RecAmountChanged(String),
    RecNoteChanged(String),
    RecDateChanged(String),
    RecKindSelected(TxKind),
    RecFreqSelected(Frequency),
    RecAccountSelected(Account),
    RecCategorySelected(Category),
    AddRecurring,
    ToggleRecurring(i64, bool),
    DeleteRecurring(i64),
    ProcessRecurring,

    // export
    ExportCsv,
    ExportPdf,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let db = Database::open_default().expect("Failed to open database");
        let mut app = Self {
            db,
            tab: Tab::Dashboard,
            status: String::new(),
            accounts: vec![],
            categories: vec![],
            transactions: vec![],
            budgets: vec![],
            recurring: vec![],
            summary: MonthSummary::default(),
            by_category: vec![],
            monthly: vec![],
            total_balance: 0,
            tx_amount: String::new(),
            tx_note: String::new(),
            tx_date: Local::now().date_naive().to_string(),
            tx_kind: TxKind::Expense,
            tx_account: None,
            tx_category: None,
            tx_to_account: None,
            acc_name: String::new(),
            acc_notes: String::new(),
            acc_opening: "0".into(),
            acc_type: AccountType::Cash,
            editing_account: None,
            cat_name: String::new(),
            cat_kind: CategoryKind::Expense,
            editing_category: None,
            budget_amount: String::new(),
            budget_category: None,
            budget_month: current_year_month(),
            rec_amount: String::new(),
            rec_note: String::new(),
            rec_date: Local::now().date_naive().to_string(),
            rec_kind: TxKind::Expense,
            rec_freq: Frequency::Monthly,
            rec_account: None,
            rec_category: None,
        };

        match app.db.process_due_recurring() {
            Ok(n) if n > 0 => {
                app.status = format!("Applied {n} due recurring transaction(s).");
            }
            Ok(_) => {}
            Err(e) => app.status = format!("Recurring error: {e}"),
        }
        app.reload();
        (app, Task::none())
    }

    fn reload(&mut self) {
        let ym = current_year_month();
        self.accounts = self.db.list_accounts().unwrap_or_default();
        self.categories = self.db.list_categories().unwrap_or_default();
        self.transactions = self.db.list_transactions(200).unwrap_or_default();
        self.budgets = self.db.list_budgets(&self.budget_month).unwrap_or_default();
        self.recurring = self.db.list_recurring().unwrap_or_default();
        self.summary = self.db.month_summary(&ym).unwrap_or_default();
        self.by_category = self.db.expense_by_category(&ym).unwrap_or_default();
        self.monthly = self.db.monthly_totals(6).unwrap_or_default();
        self.total_balance = self.db.total_balance().unwrap_or(0);

        if self.tx_account.is_none() {
            self.tx_account = self.accounts.first().cloned();
        }
        if self.tx_category.is_none() {
            self.tx_category = self
                .categories
                .iter()
                .find(|c| c.kind == CategoryKind::Expense)
                .cloned();
        }
        if self.budget_category.is_none() {
            self.budget_category = self
                .categories
                .iter()
                .find(|c| c.kind == CategoryKind::Expense)
                .cloned();
        }
        if self.rec_account.is_none() {
            self.rec_account = self.accounts.first().cloned();
        }
        if self.rec_category.is_none() {
            self.rec_category = self
                .categories
                .iter()
                .find(|c| c.kind == CategoryKind::Expense)
                .cloned();
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => {
                self.tab = tab;
                self.reload();
            }
            Message::Refresh => self.reload(),

            Message::TxAmountChanged(v) => self.tx_amount = v,
            Message::TxNoteChanged(v) => self.tx_note = v,
            Message::TxDateChanged(v) => self.tx_date = v,
            Message::TxKindSelected(k) => {
                self.tx_kind = k;
                if k != TxKind::Transfer {
                    self.tx_category = self
                        .categories
                        .iter()
                        .find(|c| {
                            matches!(
                                (k, c.kind),
                                (TxKind::Income, CategoryKind::Income)
                                    | (TxKind::Expense, CategoryKind::Expense)
                            )
                        })
                        .cloned();
                }
            }
            Message::TxAccountSelected(a) => self.tx_account = Some(a),
            Message::TxCategorySelected(c) => self.tx_category = Some(c),
            Message::TxToAccountSelected(a) => self.tx_to_account = Some(a),
            Message::AddTransaction => {
                let amount = match parse_amount(&self.tx_amount) {
                    Some(a) if a > 0 => a,
                    _ => {
                        self.status = "Enter a valid amount.".into();
                        return Task::none();
                    }
                };
                let date = match NaiveDate::parse_from_str(&self.tx_date, "%Y-%m-%d") {
                    Ok(d) => d,
                    Err(_) => {
                        self.status = "Date must be YYYY-MM-DD.".into();
                        return Task::none();
                    }
                };
                let Some(account) = self.tx_account.clone() else {
                    self.status = "Select an account.".into();
                    return Task::none();
                };
                let category = if self.tx_kind == TxKind::Transfer {
                    self.categories
                        .iter()
                        .find(|c| c.name == "Transfer")
                        .cloned()
                        .or_else(|| self.categories.first().cloned())
                } else {
                    self.tx_category.clone()
                };
                let Some(category) = category else {
                    self.status = "Select a category.".into();
                    return Task::none();
                };
                let to_id = if self.tx_kind == TxKind::Transfer {
                    match &self.tx_to_account {
                        Some(a) if a.id != account.id => Some(a.id),
                        _ => {
                            self.status = "Select a different destination account.".into();
                            return Task::none();
                        }
                    }
                } else {
                    None
                };

                match self.db.add_transaction(
                    account.id,
                    category.id,
                    amount,
                    self.tx_kind,
                    to_id,
                    date,
                    &self.tx_note,
                ) {
                    Ok(()) => {
                        self.tx_amount.clear();
                        self.tx_note.clear();
                        self.status = "Transaction saved.".into();
                        self.reload();
                    }
                    Err(e) => self.status = format!("Error: {e}"),
                }
            }
            Message::DeleteTransaction(id) => match self.db.delete_transaction(id) {
                Ok(()) => {
                    self.status = "Transaction deleted.".into();
                    self.reload();
                }
                Err(e) => self.status = format!("Error: {e}"),
            },

            Message::AccNameChanged(v) => self.acc_name = v,
            Message::AccNotesChanged(v) => self.acc_notes = v,
            Message::AccOpeningChanged(v) => self.acc_opening = v,
            Message::AccTypeSelected(t) => self.acc_type = t,
            Message::SaveAccount => {
                if self.acc_name.trim().is_empty() {
                    self.status = "Account name is required.".into();
                    return Task::none();
                }
                let opening = parse_amount(&self.acc_opening).unwrap_or(0);
                let result = if let Some(id) = self.editing_account {
                    self.db.update_account(
                        id,
                        self.acc_name.trim(),
                        self.acc_type,
                        opening,
                        &self.acc_notes,
                    )
                } else {
                    self.db.add_account(
                        self.acc_name.trim(),
                        self.acc_type,
                        opening,
                        &self.acc_notes,
                    )
                };
                match result {
                    Ok(()) => {
                        self.acc_name.clear();
                        self.acc_notes.clear();
                        self.acc_opening = "0".into();
                        self.editing_account = None;
                        self.status = "Account saved.".into();
                        self.reload();
                    }
                    Err(e) => self.status = format!("Error: {e}"),
                }
            }
            Message::EditAccount(id) => {
                if let Some(a) = self.accounts.iter().find(|a| a.id == id) {
                    self.editing_account = Some(id);
                    self.acc_name = a.name.clone();
                    self.acc_notes = a.notes.clone();
                    self.acc_opening = a.opening_balance.to_string();
                    self.acc_type = a.account_type;
                }
            }
            Message::CancelEditAccount => {
                self.editing_account = None;
                self.acc_name.clear();
                self.acc_notes.clear();
                self.acc_opening = "0".into();
            }
            Message::DeleteAccount(id) => match self.db.delete_account(id) {
                Ok(()) => {
                    self.status = "Account deleted.".into();
                    self.reload();
                }
                Err(e) => self.status = format!("Error: {e}"),
            },

            Message::CatNameChanged(v) => self.cat_name = v,
            Message::CatKindSelected(k) => self.cat_kind = k,
            Message::SaveCategory => {
                if self.cat_name.trim().is_empty() {
                    self.status = "Category name is required.".into();
                    return Task::none();
                }
                let result = if let Some(id) = self.editing_category {
                    self.db
                        .update_category(id, self.cat_name.trim(), self.cat_kind)
                } else {
                    self.db.add_category(self.cat_name.trim(), self.cat_kind)
                };
                match result {
                    Ok(()) => {
                        self.cat_name.clear();
                        self.editing_category = None;
                        self.status = "Category saved.".into();
                        self.reload();
                    }
                    Err(e) => self.status = format!("Error: {e}"),
                }
            }
            Message::EditCategory(id) => {
                if let Some(c) = self.categories.iter().find(|c| c.id == id) {
                    self.editing_category = Some(id);
                    self.cat_name = c.name.clone();
                    self.cat_kind = c.kind;
                }
            }
            Message::CancelEditCategory => {
                self.editing_category = None;
                self.cat_name.clear();
            }
            Message::DeleteCategory(id) => match self.db.delete_category(id) {
                Ok(()) => {
                    self.status = "Category deleted.".into();
                    self.reload();
                }
                Err(e) => {
                    self.status =
                        format!("Cannot delete (in use?): {e}");
                }
            },

            Message::BudgetAmountChanged(v) => self.budget_amount = v,
            Message::BudgetCategorySelected(c) => self.budget_category = Some(c),
            Message::BudgetMonthChanged(v) => {
                self.budget_month = v;
                self.budgets = self.db.list_budgets(&self.budget_month).unwrap_or_default();
            }
            Message::SaveBudget => {
                let amount = match parse_amount(&self.budget_amount) {
                    Some(a) if a > 0 => a,
                    _ => {
                        self.status = "Enter a valid budget limit.".into();
                        return Task::none();
                    }
                };
                let Some(cat) = self.budget_category.clone() else {
                    self.status = "Select a category.".into();
                    return Task::none();
                };
                match self
                    .db
                    .upsert_budget(cat.id, &self.budget_month, amount)
                {
                    Ok(()) => {
                        self.budget_amount.clear();
                        self.status = "Budget saved.".into();
                        self.reload();
                    }
                    Err(e) => self.status = format!("Error: {e}"),
                }
            }
            Message::DeleteBudget(id) => match self.db.delete_budget(id) {
                Ok(()) => {
                    self.status = "Budget deleted.".into();
                    self.reload();
                }
                Err(e) => self.status = format!("Error: {e}"),
            },

            Message::RecAmountChanged(v) => self.rec_amount = v,
            Message::RecNoteChanged(v) => self.rec_note = v,
            Message::RecDateChanged(v) => self.rec_date = v,
            Message::RecKindSelected(k) => self.rec_kind = k,
            Message::RecFreqSelected(f) => self.rec_freq = f,
            Message::RecAccountSelected(a) => self.rec_account = Some(a),
            Message::RecCategorySelected(c) => self.rec_category = Some(c),
            Message::AddRecurring => {
                let amount = match parse_amount(&self.rec_amount) {
                    Some(a) if a > 0 => a,
                    _ => {
                        self.status = "Enter a valid amount.".into();
                        return Task::none();
                    }
                };
                let date = match NaiveDate::parse_from_str(&self.rec_date, "%Y-%m-%d") {
                    Ok(d) => d,
                    Err(_) => {
                        self.status = "Date must be YYYY-MM-DD.".into();
                        return Task::none();
                    }
                };
                let Some(account) = self.rec_account.clone() else {
                    self.status = "Select an account.".into();
                    return Task::none();
                };
                let Some(category) = self.rec_category.clone() else {
                    self.status = "Select a category.".into();
                    return Task::none();
                };
                match self.db.add_recurring(
                    account.id,
                    category.id,
                    amount,
                    self.rec_kind,
                    self.rec_freq,
                    date,
                    &self.rec_note,
                ) {
                    Ok(()) => {
                        self.rec_amount.clear();
                        self.rec_note.clear();
                        self.status = "Recurring rule added.".into();
                        self.reload();
                    }
                    Err(e) => self.status = format!("Error: {e}"),
                }
            }
            Message::ToggleRecurring(id, active) => {
                match self.db.set_recurring_active(id, active) {
                    Ok(()) => {
                        self.reload();
                    }
                    Err(e) => self.status = format!("Error: {e}"),
                }
            }
            Message::DeleteRecurring(id) => match self.db.delete_recurring(id) {
                Ok(()) => {
                    self.status = "Recurring rule deleted.".into();
                    self.reload();
                }
                Err(e) => self.status = format!("Error: {e}"),
            },
            Message::ProcessRecurring => match self.db.process_due_recurring() {
                Ok(n) => {
                    self.status = format!("Processed {n} recurring transaction(s).");
                    self.reload();
                }
                Err(e) => self.status = format!("Error: {e}"),
            },

            Message::ExportCsv => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Export CSV")
                    .set_file_name("transactions.csv")
                    .add_filter("CSV", &["csv"])
                    .save_file()
                {
                    match self.db.export_transactions_csv(&path) {
                        Ok(n) => {
                            self.status = format!("Exported {n} transactions to CSV.");
                        }
                        Err(e) => self.status = format!("CSV export failed: {e}"),
                    }
                }
            }
            Message::ExportPdf => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Export PDF Report")
                    .set_file_name("finance-report.pdf")
                    .add_filter("PDF", &["pdf"])
                    .save_file()
                {
                    match export::export_monthly_pdf(&self.db, &path) {
                        Ok(()) => self.status = "PDF report exported.".into(),
                        Err(e) => self.status = format!("PDF export failed: {e}"),
                    }
                }
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let title = container(
            row![
                title_text("Finance Manager"),
                Space::new().width(Fill),
                title_text("Currency: IDR"),
            ]
            .align_y(Alignment::Center)
            .padding([4, 8]),
        )
        .style(title_bar)
        .width(Fill);

        let tabs = row(Tab::ALL.iter().map(|t| {
            button(text(t.label()).size(12))
                .padding([6, 10])
                .style(xp_tab(self.tab == *t))
                .on_press(Message::SwitchTab(*t))
                .into()
        }))
        .spacing(2)
        .padding([4, 4]);

        let body = match self.tab {
            Tab::Dashboard => self.view_dashboard(),
            Tab::Transactions => self.view_transactions(),
            Tab::Accounts => self.view_accounts(),
            Tab::Categories => self.view_categories(),
            Tab::Budgets => self.view_budgets(),
            Tab::Recurring => self.view_recurring(),
            Tab::Reports => self.view_reports(),
            Tab::Export => self.view_export(),
        };

        let status = container(
            row![
                label_text(if self.status.is_empty() {
                    "Ready.".into()
                } else {
                    self.status.clone()
                }),
                Space::new().width(Fill),
                button(text("Refresh").size(12))
                    .padding([4, 10])
                    .style(xp_button)
                    .on_press(Message::Refresh),
            ]
            .align_y(Alignment::Center),
        )
        .padding(6)
        .style(face_container)
        .width(Fill);

        container(
            column![title, tabs, body, status]
                .spacing(0)
                .width(Fill)
                .height(Fill),
        )
        .style(face_container)
        .width(Fill)
        .height(Fill)
        .into()
    }

    fn view_dashboard(&self) -> Element<'_, Message> {
        let cards = row![
            Self::stat_box("Total Balance", format_idr(self.total_balance), XP_TEXT),
            Self::stat_box(
                "Income (This Month)",
                format_idr(self.summary.income),
                XP_SUCCESS
            ),
            Self::stat_box(
                "Expense (This Month)",
                format_idr(self.summary.expense),
                XP_DANGER
            ),
            Self::stat_box(
                "Net (This Month)",
                format_idr(self.summary.net()),
                XP_TEXT
            ),
        ]
        .spacing(8);

        let recent = column(
            std::iter::once(section_header("Recent Transactions").into()).chain(
                self.transactions.iter().take(8).map(|t| {
                    label_text(format!(
                        "{} | {} | {} | {} | {}",
                        t.date,
                        t.kind,
                        t.account_name,
                        t.category_name,
                        format_idr(t.amount)
                    ))
                    .into()
                }),
            ),
        )
        .spacing(4);

        let chart = container(
            column![
                section_header("Last 6 Months"),
                BarChart {
                    data: self.monthly.clone()
                }
                .into_element()
                .map(|_| Message::Refresh),
            ]
            .spacing(6),
        )
        .padding(8)
        .style(white_panel)
        .width(Fill);

        scrollable(
            container(
                column![
                    section_header(format!("Dashboard — {}", current_year_month())),
                    cards,
                    chart,
                    container(recent).padding(8).style(white_panel).width(Fill),
                ]
                .spacing(10)
                .padding(10),
            )
            .width(Fill),
        )
        .height(Fill)
        .into()
    }

    fn stat_box(title: &str, value: String, color: iced::Color) -> Element<'static, Message> {
        container(
            column![
                text(title.to_string()).size(12).color(XP_TEXT),
                text(value).size(16).color(color),
            ]
            .spacing(4),
        )
        .padding(10)
        .style(white_panel)
        .width(Fill)
        .into()
    }

    fn view_transactions(&self) -> Element<'_, Message> {
        let kind_filter_cats: Vec<Category> = self
            .categories
            .iter()
            .filter(|c| match self.tx_kind {
                TxKind::Income => c.kind == CategoryKind::Income,
                TxKind::Expense => c.kind == CategoryKind::Expense,
                TxKind::Transfer => true,
            })
            .cloned()
            .collect();

        let form = container(
            column![
                section_header("Add Transaction"),
                row![
                    field(
                        "Type",
                        pick_list(TxKind::ALL, Some(self.tx_kind), Message::TxKindSelected)
                            .width(120),
                    ),
                    field(
                        "Amount (IDR)",
                        text_input("e.g. 50000", &self.tx_amount)
                            .on_input(Message::TxAmountChanged)
                            .style(xp_input)
                            .width(140),
                    ),
                    field(
                        "Date",
                        text_input("YYYY-MM-DD", &self.tx_date)
                            .on_input(Message::TxDateChanged)
                            .style(xp_input)
                            .width(120),
                    ),
                ]
                .spacing(8),
                row![
                    field(
                        "Account",
                        pick_list(
                            self.accounts.clone(),
                            self.tx_account.clone(),
                            Message::TxAccountSelected
                        )
                        .width(160),
                    ),
                    if self.tx_kind == TxKind::Transfer {
                        field(
                            "To Account",
                            pick_list(
                                self.accounts.clone(),
                                self.tx_to_account.clone(),
                                Message::TxToAccountSelected
                            )
                            .width(160),
                        )
                    } else {
                        field(
                            "Category",
                            pick_list(
                                kind_filter_cats,
                                self.tx_category.clone(),
                                Message::TxCategorySelected
                            )
                            .width(160),
                        )
                    },
                    field(
                        "Note",
                        text_input("Optional note", &self.tx_note)
                            .on_input(Message::TxNoteChanged)
                            .style(xp_input)
                            .width(200),
                    ),
                    column![
                        label_text(" "),
                        button(text("Add").size(13))
                            .padding([6, 16])
                            .style(xp_button)
                            .on_press(Message::AddTransaction),
                    ]
                    .spacing(2),
                ]
                .spacing(8)
                .align_y(Alignment::End),
            ]
            .spacing(8),
        )
        .padding(10)
        .style(white_panel)
        .width(Fill);

        let list = container(
            column(
                std::iter::once(section_header("All Transactions").into()).chain(
                    self.transactions.iter().map(|t| {
                        row![
                            label_text(format!(
                                "{}  {}  {} → {}  {}  {}",
                                t.date,
                                t.kind,
                                t.account_name,
                                t.category_name,
                                format_idr(t.amount),
                                if t.note.is_empty() {
                                    String::new()
                                } else {
                                    format!("({})", t.note)
                                }
                            ))
                            .width(Fill),
                            button(text("Delete").size(11))
                                .padding([3, 8])
                                .style(xp_button)
                                .on_press(Message::DeleteTransaction(t.id)),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .into()
                    }),
                ),
            )
            .spacing(4),
        )
        .padding(10)
        .style(inset_panel)
        .width(Fill);

        scrollable(
            column![form, list].spacing(10).padding(10),
        )
        .height(Fill)
        .into()
    }

    fn view_accounts(&self) -> Element<'_, Message> {
        let form = container(
            column![
                section_header(if self.editing_account.is_some() {
                    "Edit Account"
                } else {
                    "New Account"
                }),
                row![
                    field(
                        "Name",
                        text_input("Account name", &self.acc_name)
                            .on_input(Message::AccNameChanged)
                            .style(xp_input)
                            .width(160),
                    ),
                    field(
                        "Type",
                        pick_list(
                            AccountType::ALL,
                            Some(self.acc_type),
                            Message::AccTypeSelected
                        )
                        .width(130),
                    ),
                    field(
                        "Opening Balance",
                        text_input("0", &self.acc_opening)
                            .on_input(Message::AccOpeningChanged)
                            .style(xp_input)
                            .width(140),
                    ),
                    field(
                        "Notes",
                        text_input("Notes", &self.acc_notes)
                            .on_input(Message::AccNotesChanged)
                            .style(xp_input)
                            .width(180),
                    ),
                    column![
                        label_text(" "),
                        row![
                            button(text("Save").size(13))
                                .padding([6, 14])
                                .style(xp_button)
                                .on_press(Message::SaveAccount),
                            button(text("Cancel").size(13))
                                .padding([6, 14])
                                .style(xp_button)
                                .on_press(Message::CancelEditAccount),
                        ]
                        .spacing(4),
                    ]
                    .spacing(2),
                ]
                .spacing(8)
                .align_y(Alignment::End),
            ]
            .spacing(8),
        )
        .padding(10)
        .style(white_panel)
        .width(Fill);

        let list = container(
            column(
                std::iter::once(section_header("Accounts").into()).chain(
                    self.accounts.iter().map(|a| {
                        row![
                            label_text(format!(
                                "{}  [{}]  Balance: {}  Opening: {}",
                                a.name,
                                a.account_type,
                                format_idr(a.balance),
                                format_idr(a.opening_balance)
                            ))
                            .width(Fill),
                            button(text("Edit").size(11))
                                .padding([3, 8])
                                .style(xp_button)
                                .on_press(Message::EditAccount(a.id)),
                            button(text("Delete").size(11))
                                .padding([3, 8])
                                .style(xp_button)
                                .on_press(Message::DeleteAccount(a.id)),
                        ]
                        .spacing(6)
                        .into()
                    }),
                ),
            )
            .spacing(4),
        )
        .padding(10)
        .style(inset_panel)
        .width(Fill);

        scrollable(column![form, list].spacing(10).padding(10))
            .height(Fill)
            .into()
    }

    fn view_categories(&self) -> Element<'_, Message> {
        let form = container(
            column![
                section_header(if self.editing_category.is_some() {
                    "Edit Category"
                } else {
                    "New Category"
                }),
                row![
                    field(
                        "Name",
                        text_input("Category name", &self.cat_name)
                            .on_input(Message::CatNameChanged)
                            .style(xp_input)
                            .width(200),
                    ),
                    field(
                        "Kind",
                        pick_list(
                            CategoryKind::ALL,
                            Some(self.cat_kind),
                            Message::CatKindSelected
                        )
                        .width(120),
                    ),
                    column![
                        label_text(" "),
                        row![
                            button(text("Save").size(13))
                                .padding([6, 14])
                                .style(xp_button)
                                .on_press(Message::SaveCategory),
                            button(text("Cancel").size(13))
                                .padding([6, 14])
                                .style(xp_button)
                                .on_press(Message::CancelEditCategory),
                        ]
                        .spacing(4),
                    ]
                    .spacing(2),
                ]
                .spacing(8)
                .align_y(Alignment::End),
            ]
            .spacing(8),
        )
        .padding(10)
        .style(white_panel)
        .width(Fill);

        let list = container(
            column(
                std::iter::once(section_header("Categories").into()).chain(
                    self.categories.iter().map(|c| {
                        row![
                            label_text(format!("{}  [{}]", c.name, c.kind)).width(Fill),
                            button(text("Edit").size(11))
                                .padding([3, 8])
                                .style(xp_button)
                                .on_press(Message::EditCategory(c.id)),
                            button(text("Delete").size(11))
                                .padding([3, 8])
                                .style(xp_button)
                                .on_press(Message::DeleteCategory(c.id)),
                        ]
                        .spacing(6)
                        .into()
                    }),
                ),
            )
            .spacing(4),
        )
        .padding(10)
        .style(inset_panel)
        .width(Fill);

        scrollable(column![form, list].spacing(10).padding(10))
            .height(Fill)
            .into()
    }

    fn view_budgets(&self) -> Element<'_, Message> {
        let expense_cats: Vec<Category> = self
            .categories
            .iter()
            .filter(|c| c.kind == CategoryKind::Expense)
            .cloned()
            .collect();

        let form = container(
            column![
                section_header("Set Monthly Budget"),
                row![
                    field(
                        "Month (YYYY-MM)",
                        text_input("YYYY-MM", &self.budget_month)
                            .on_input(Message::BudgetMonthChanged)
                            .style(xp_input)
                            .width(120),
                    ),
                    field(
                        "Category",
                        pick_list(
                            expense_cats,
                            self.budget_category.clone(),
                            Message::BudgetCategorySelected
                        )
                        .width(160),
                    ),
                    field(
                        "Limit (IDR)",
                        text_input("e.g. 1000000", &self.budget_amount)
                            .on_input(Message::BudgetAmountChanged)
                            .style(xp_input)
                            .width(140),
                    ),
                    column![
                        label_text(" "),
                        button(text("Save Budget").size(13))
                            .padding([6, 14])
                            .style(xp_button)
                            .on_press(Message::SaveBudget),
                    ]
                    .spacing(2),
                ]
                .spacing(8)
                .align_y(Alignment::End),
            ]
            .spacing(8),
        )
        .padding(10)
        .style(white_panel)
        .width(Fill);

        let list = container(
            column(
                std::iter::once(
                    section_header(format!("Budgets — {}", self.budget_month)).into(),
                )
                .chain(self.budgets.iter().map(|b| {
                    let pct = if b.limit_amount > 0 {
                        (b.spent as f64 / b.limit_amount as f64 * 100.0).min(999.0)
                    } else {
                        0.0
                    };
                    let over = b.spent > b.limit_amount;
                    row![
                        label_text(format!(
                            "{}  Spent {} / {}  ({:.0}%){}",
                            b.category_name,
                            format_idr(b.spent),
                            format_idr(b.limit_amount),
                            pct,
                            if over { "  OVER BUDGET" } else { "" }
                        ))
                        .color(if over { XP_DANGER } else { XP_TEXT })
                        .width(Fill),
                        button(text("Delete").size(11))
                            .padding([3, 8])
                            .style(xp_button)
                            .on_press(Message::DeleteBudget(b.id)),
                    ]
                    .spacing(6)
                    .into()
                })),
            )
            .spacing(4),
        )
        .padding(10)
        .style(inset_panel)
        .width(Fill);

        scrollable(column![form, list].spacing(10).padding(10))
            .height(Fill)
            .into()
    }

    fn view_recurring(&self) -> Element<'_, Message> {
        let form = container(
            column![
                section_header("New Recurring Rule"),
                row![
                    field(
                        "Type",
                        pick_list(
                            [TxKind::Income, TxKind::Expense],
                            Some(self.rec_kind),
                            Message::RecKindSelected
                        )
                        .width(120),
                    ),
                    field(
                        "Amount",
                        text_input("IDR", &self.rec_amount)
                            .on_input(Message::RecAmountChanged)
                            .style(xp_input)
                            .width(120),
                    ),
                    field(
                        "Frequency",
                        pick_list(
                            Frequency::ALL,
                            Some(self.rec_freq),
                            Message::RecFreqSelected
                        )
                        .width(120),
                    ),
                    field(
                        "Next Date",
                        text_input("YYYY-MM-DD", &self.rec_date)
                            .on_input(Message::RecDateChanged)
                            .style(xp_input)
                            .width(120),
                    ),
                ]
                .spacing(8),
                row![
                    field(
                        "Account",
                        pick_list(
                            self.accounts.clone(),
                            self.rec_account.clone(),
                            Message::RecAccountSelected
                        )
                        .width(160),
                    ),
                    field(
                        "Category",
                        pick_list(
                            self.categories.clone(),
                            self.rec_category.clone(),
                            Message::RecCategorySelected
                        )
                        .width(160),
                    ),
                    field(
                        "Note",
                        text_input("Note", &self.rec_note)
                            .on_input(Message::RecNoteChanged)
                            .style(xp_input)
                            .width(180),
                    ),
                    column![
                        label_text(" "),
                        button(text("Add Rule").size(13))
                            .padding([6, 14])
                            .style(xp_button)
                            .on_press(Message::AddRecurring),
                    ]
                    .spacing(2),
                ]
                .spacing(8)
                .align_y(Alignment::End),
                button(text("Process Due Now").size(12))
                    .padding([6, 12])
                    .style(xp_button)
                    .on_press(Message::ProcessRecurring),
            ]
            .spacing(8),
        )
        .padding(10)
        .style(white_panel)
        .width(Fill);

        let list = container(
            column(
                std::iter::once(section_header("Recurring Rules").into()).chain(
                    self.recurring.iter().map(|r| {
                        row![
                            label_text(format!(
                                "[{}] {} {} {} via {} / {} next {} — {}",
                                if r.active { "ON" } else { "OFF" },
                                r.kind,
                                format_idr(r.amount),
                                r.frequency,
                                r.account_name,
                                r.category_name,
                                r.next_date,
                                r.note
                            ))
                            .width(Fill),
                            button(text(if r.active { "Disable" } else { "Enable" }).size(11))
                                .padding([3, 8])
                                .style(xp_button)
                                .on_press(Message::ToggleRecurring(r.id, !r.active)),
                            button(text("Delete").size(11))
                                .padding([3, 8])
                                .style(xp_button)
                                .on_press(Message::DeleteRecurring(r.id)),
                        ]
                        .spacing(6)
                        .into()
                    }),
                ),
            )
            .spacing(4),
        )
        .padding(10)
        .style(inset_panel)
        .width(Fill);

        scrollable(column![form, list].spacing(10).padding(10))
            .height(Fill)
            .into()
    }

    fn view_reports(&self) -> Element<'_, Message> {
        let cat_lines = column(
            std::iter::once(section_header("Expenses by Category (This Month)").into())
                .chain(self.by_category.iter().map(|c| {
                    label_text(format!("{} — {}", c.name, format_idr(c.amount))).into()
                })),
        )
        .spacing(4);

        scrollable(
            column![
                section_header(format!("Reports — {}", current_year_month())),
                container(
                    column![
                        label_text(format!("Income:  {}", format_idr(self.summary.income))),
                        label_text(format!("Expense: {}", format_idr(self.summary.expense))),
                        label_text(format!("Net:     {}", format_idr(self.summary.net()))),
                        label_text(format!(
                            "Total Balance: {}",
                            format_idr(self.total_balance)
                        )),
                    ]
                    .spacing(4),
                )
                .padding(10)
                .style(white_panel)
                .width(Fill),
                container(
                    column![
                        section_header("Income vs Expense (6 Months)"),
                        BarChart {
                            data: self.monthly.clone()
                        }
                        .into_element()
                        .map(|_| Message::Refresh),
                    ]
                    .spacing(6),
                )
                .padding(10)
                .style(white_panel)
                .width(Fill),
                container(cat_lines)
                    .padding(10)
                    .style(inset_panel)
                    .width(Fill),
            ]
            .spacing(10)
            .padding(10),
        )
        .height(Fill)
        .into()
    }

    fn view_export(&self) -> Element<'_, Message> {
        container(
            column![
                section_header("Export Data"),
                label_text(
                    "Export all transactions to CSV, or generate a monthly PDF summary report.",
                ),
                Space::new().height(12),
                row![
                    button(text("Export Transactions (CSV)").size(13))
                        .padding([8, 16])
                        .style(xp_button)
                        .on_press(Message::ExportCsv),
                    button(text("Export Monthly Report (PDF)").size(13))
                        .padding([8, 16])
                        .style(xp_button)
                        .on_press(Message::ExportPdf),
                ]
                .spacing(10),
                Space::new().height(16),
                label_text(format!(
                    "Database location is under your OS app data folder (FinanceManager)."
                )),
            ]
            .spacing(6),
        )
        .padding(16)
        .style(white_panel)
        .width(Fill)
        .height(Fill)
        .into()
    }
}

// Mine — labeled form field
fn field<'a>(label: &'a str, child: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    column![label_text(label.to_string()), child.into()]
        .spacing(2)
        .into()
}
