// Mine — Simple XP-style bar chart via iced canvas

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Element, Point, Rectangle, Renderer, Size, Theme};

use crate::models::MonthSummary;
use crate::theme::{XP_DANGER, XP_SUCCESS, XP_TEXT, XP_TITLE};
use crate::util::format_idr;

#[derive(Debug, Clone)]
pub struct BarChart {
    pub data: Vec<(String, MonthSummary)>,
}

impl BarChart {
    pub fn into_element(self) -> Element<'static, ()> {
        Canvas::new(self).width(iced::Fill).height(220).into()
    }
}

impl<Message> canvas::Program<Message> for BarChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        frame.fill(
            &Path::rectangle(Point::ORIGIN, bounds.size()),
            Color::WHITE,
        );

        if self.data.is_empty() {
            frame.fill_text(Text {
                content: "No data yet".into(),
                position: Point::new(bounds.width / 2.0, bounds.height / 2.0),
                color: XP_TEXT,
                size: iced::Pixels(13.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                ..Text::default()
            });
            return vec![frame.into_geometry()];
        }

        let max_val = self
            .data
            .iter()
            .map(|(_, s)| s.income.max(s.expense))
            .max()
            .unwrap_or(1)
            .max(1) as f32;

        let pad_l = 48.0;
        let pad_b = 36.0;
        let pad_t = 16.0;
        let pad_r = 12.0;
        let chart_w = (bounds.width - pad_l - pad_r).max(10.0);
        let chart_h = (bounds.height - pad_t - pad_b).max(10.0);
        let n = self.data.len() as f32;
        let group_w = chart_w / n;
        let bar_w = (group_w * 0.32).clamp(6.0, 28.0);

        let axis = Path::line(
            Point::new(pad_l, pad_t),
            Point::new(pad_l, pad_t + chart_h),
        );
        frame.stroke(
            &axis,
            Stroke::default().with_width(1.0).with_color(XP_TITLE),
        );
        let axis2 = Path::line(
            Point::new(pad_l, pad_t + chart_h),
            Point::new(pad_l + chart_w, pad_t + chart_h),
        );
        frame.stroke(
            &axis2,
            Stroke::default().with_width(1.0).with_color(XP_TITLE),
        );

        for (i, (label, summary)) in self.data.iter().enumerate() {
            let cx = pad_l + group_w * (i as f32 + 0.5);
            let income_h = (summary.income as f32 / max_val) * chart_h;
            let expense_h = (summary.expense as f32 / max_val) * chart_h;

            let income_rect = Path::rectangle(
                Point::new(cx - bar_w - 2.0, pad_t + chart_h - income_h),
                Size::new(bar_w, income_h),
            );
            frame.fill(&income_rect, XP_SUCCESS);

            let expense_rect = Path::rectangle(
                Point::new(cx + 2.0, pad_t + chart_h - expense_h),
                Size::new(bar_w, expense_h),
            );
            frame.fill(&expense_rect, XP_DANGER);

            let short = if label.len() >= 7 {
                &label[2..]
            } else {
                label.as_str()
            };
            frame.fill_text(Text {
                content: short.into(),
                position: Point::new(cx, pad_t + chart_h + 12.0),
                color: XP_TEXT,
                size: iced::Pixels(11.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Top,
                ..Text::default()
            });
        }

        frame.fill(
            &Path::rectangle(Point::new(pad_l + 8.0, 4.0), Size::new(10.0, 10.0)),
            XP_SUCCESS,
        );
        frame.fill_text(Text {
            content: "Income".into(),
            position: Point::new(pad_l + 22.0, 9.0),
            color: XP_TEXT,
            size: iced::Pixels(11.0),
            align_y: iced::alignment::Vertical::Center,
            ..Text::default()
        });
        frame.fill(
            &Path::rectangle(Point::new(pad_l + 80.0, 4.0), Size::new(10.0, 10.0)),
            XP_DANGER,
        );
        frame.fill_text(Text {
            content: format!("Expense  (max {})", format_idr(max_val as i64)),
            position: Point::new(pad_l + 94.0, 9.0),
            color: XP_TEXT,
            size: iced::Pixels(11.0),
            align_y: iced::alignment::Vertical::Center,
            ..Text::default()
        });

        vec![frame.into_geometry()]
    }
}
