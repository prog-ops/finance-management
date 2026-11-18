// Mine — Formatting helpers (IDR)

use chrono::Datelike;

/// Format an integer IDR amount with thousand separators, e.g. Rp 1.250.000
pub fn format_idr(amount: i64) -> String {
    let neg = amount < 0;
    let n = amount.unsigned_abs();
    let raw = n.to_string();
    let mut out = String::new();
    for (i, ch) in raw.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('.');
        }
        out.push(ch);
    }
    let digits: String = out.chars().rev().collect();
    if neg {
        format!("-Rp {digits}")
    } else {
        format!("Rp {digits}")
    }
}

pub fn parse_amount(input: &str) -> Option<i64> {
    let cleaned: String = input
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse().ok()
}

pub fn current_year_month() -> String {
    let now = chrono::Local::now().date_naive();
    format!("{:04}-{:02}", now.year(), now.month())
}
