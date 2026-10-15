//! Dense data table with sortable headers and selectable rows.

use iced::widget::{self, button, column, container, row, rule, text};
use iced::{Alignment, Element, Length, Padding, alignment::Horizontal};
use yaams_api::SortDirection;

use crate::icons::{Icon, icon};
use crate::theme;

pub const ROW_HEIGHT: f32 = 38.0;
const CELL_PADDING: Padding = Padding {
    top: 0.0,
    right: 12.0,
    bottom: 0.0,
    left: 12.0,
};

#[derive(Debug, Clone, Copy)]
pub struct Column<S> {
    pub title: &'static str,
    /// Relative width (fill portion).
    pub width: u16,
    pub sort: Option<S>,
    pub align_right: bool,
}

impl<S> Column<S> {
    pub const fn new(title: &'static str, width: u16, sort: Option<S>) -> Self {
        Self {
            title,
            width,
            sort,
            align_right: false,
        }
    }

    pub const fn right(mut self) -> Self {
        self.align_right = true;
        self
    }
}

fn cell<'a, M: 'a>(content: impl Into<Element<'a, M>>, column: &Column<impl Copy>) -> Element<'a, M> {
    container(content)
        .padding(CELL_PADDING)
        .width(Length::FillPortion(column.width))
        .align_x(if column.align_right {
            Horizontal::Right
        } else {
            Horizontal::Left
        })
        .into()
}

/// Header row. Sortable columns show the active direction and toggle on click.
pub fn header<'a, M: Clone + 'a, S: Copy + PartialEq + 'a>(
    columns: &[Column<S>],
    current: (S, SortDirection),
    on_sort: impl Fn(S) -> M,
) -> Element<'a, M> {
    let cells = columns.iter().map(|column| {
        let title = text(column.title.to_uppercase()).size(12).font(theme::FONT_CAPTION);
        let content: Element<'a, M> = match column.sort {
            Some(key) => {
                let active = key == current.0;
                let glyph = match (active, current.1) {
                    (true, SortDirection::Ascending) => Icon::ChevronUp,
                    (true, SortDirection::Descending) => Icon::ChevronDown,
                    _ => Icon::ChevronsUpDown,
                };
                let indicator = icon(glyph, 12).color(if active { theme::AMBER } else { theme::TEXT_MUTED });
                let title = if active { title.color(theme::TEXT) } else { title };
                button(row![title, indicator].spacing(4).align_y(Alignment::Center))
                    .padding(0)
                    .style(theme::table_header_cell)
                    .on_press(on_sort(key))
                    .into()
            }
            None => title.into(),
        };
        cell(content, column)
    });
    container(row(cells).align_y(Alignment::Center).height(Length::Fixed(36.0)))
        .width(Length::Fill)
        .style(theme::table_header)
        .into()
}

/// A selectable data row.
pub fn data_row<'a, M: Clone + 'a, S: Copy>(
    columns: &[Column<S>],
    cells: Vec<Element<'a, M>>,
    selected: bool,
    on_press: M,
) -> Element<'a, M> {
    let cells = columns.iter().zip(cells).map(|(column, content)| cell(content, column));
    button(row(cells).align_y(Alignment::Center).height(Length::Fixed(ROW_HEIGHT)))
        .padding(0)
        .width(Length::Fill)
        .style(theme::table_row(selected))
        .on_press(on_press)
        .into()
}

/// Stacks rows ledger-style, with a hairline rule under each one.
pub fn ruled<'a, M: 'a>(rows: impl IntoIterator<Item = Element<'a, M>>) -> widget::Column<'a, M> {
    rows.into_iter().fold(column![], |col, row| {
        col.push(row).push(rule::horizontal(1).style(theme::divider))
    })
}
