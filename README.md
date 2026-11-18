# Finance Manager

Personal finance desktop app built with **Rust** + **iced**, styled like classic **Windows XP**, using **Tahoma**.

Runs on **Windows** and **macOS**. All UI text is in English. Default currency is **IDR**.

## Features

- Income, expense, and transfer tracking
- Multiple accounts (Cash, Bank, E-Wallet, Investment, Other)
- Categories
- Monthly budgets with overspend warnings
- Recurring transactions (daily / weekly / monthly / yearly)
- Dashboard + bar charts (last 6 months)
- CSV export and PDF monthly report
- Local SQLite database (no cloud)

## Requirements

- Rust 1.88+ (edition 2024)
- On macOS: install Tahoma if you want the authentic XP look (otherwise the system falls back to a similar sans-serif)

## Run

```bash
cargo run --release
```

## Data location

SQLite file is stored in the OS app data directory for `FinanceManager` (via the `directories` crate).
