// Mine — PDF report export (printpdf 0.7 API)

use anyhow::{Context, Result};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::db::Database;
use crate::util::{current_year_month, format_idr};

pub fn export_monthly_pdf(db: &Database, path: &Path) -> Result<()> {
    let ym = current_year_month();
    let summary = db.month_summary(&ym)?;
    let by_cat = db.expense_by_category(&ym)?;
    let accounts = db.list_accounts()?;
    let total = db.total_balance()?;

    let (doc, page1, layer1) =
        PdfDocument::new("Finance Manager Report", Mm(210.0), Mm(297.0), "Layer 1");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let layer = doc.get_page(page1).get_layer(layer1);

    let mut y = 280.0;

    layer.use_text(
        "Personal Finance Report",
        18.0,
        Mm(20.0),
        Mm(y),
        &font_bold,
    );
    y -= 10.0;
    layer.use_text(
        format!("Period: {ym}  |  Currency: IDR"),
        11.0,
        Mm(20.0),
        Mm(y),
        &font,
    );
    y -= 12.0;

    layer.use_text("Summary", 14.0, Mm(20.0), Mm(y), &font_bold);
    y -= 8.0;
    for line in [
        format!("Total Balance: {}", format_idr(total)),
        format!("Income:        {}", format_idr(summary.income)),
        format!("Expense:       {}", format_idr(summary.expense)),
        format!("Net:           {}", format_idr(summary.net())),
    ] {
        layer.use_text(line, 11.0, Mm(25.0), Mm(y), &font);
        y -= 6.0;
    }

    y -= 6.0;
    layer.use_text("Accounts", 14.0, Mm(20.0), Mm(y), &font_bold);
    y -= 8.0;
    for a in &accounts {
        layer.use_text(
            format!(
                "{} ({}) — {}",
                a.name,
                a.account_type.as_str(),
                format_idr(a.balance)
            ),
            10.0,
            Mm(25.0),
            Mm(y),
            &font,
        );
        y -= 5.5;
        if y < 30.0 {
            break;
        }
    }

    y -= 6.0;
    if y > 40.0 {
        layer.use_text("Expenses by Category", 14.0, Mm(20.0), Mm(y), &font_bold);
        y -= 8.0;
        for c in &by_cat {
            layer.use_text(
                format!("{} — {}", c.name, format_idr(c.amount)),
                10.0,
                Mm(25.0),
                Mm(y),
                &font,
            );
            y -= 5.5;
            if y < 20.0 {
                break;
            }
        }
    }

    doc.save(&mut BufWriter::new(
        File::create(path).with_context(|| format!("Cannot create {}", path.display()))?,
    ))?;
    Ok(())
}
