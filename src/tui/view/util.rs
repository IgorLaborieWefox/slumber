//! Helper structs and functions for building components

use crate::{
    config::{Profile, RequestRecipe},
    tui::view::{
        state::{FixedSelect, Notification, StatefulList, StatefulSelect},
        RenderContext,
    },
};
use chrono::{DateTime, Duration, Local, Utc};
use indexmap::IndexMap;
use ratatui::{
    prelude::*,
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Tabs},
};
use reqwest::header::HeaderMap;
use std::fmt::Display;

/// A helper for building a UI. It can be converted into some UI element to be
/// drawn.
pub trait ToTui {
    type Output<'this>
    where
        Self: 'this;

    /// Build a UI element
    fn to_tui(&self, context: &RenderContext) -> Self::Output<'_>;
}

/// A container with a title and border
pub struct BlockBrick {
    pub title: String,
    pub is_focused: bool,
}

impl ToTui for BlockBrick {
    type Output<'this> = Block<'this> where Self: 'this;

    fn to_tui(&self, context: &RenderContext) -> Self::Output<'_> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(context.theme.pane_border_style(self.is_focused))
            .title(self.title.as_str())
    }
}

/// A piece of text that looks interactable
pub struct ButtonBrick<'a> {
    pub text: &'a str,
    pub is_highlighted: bool,
}

impl<'a> ToTui for ButtonBrick<'a> {
    type Output<'this> = Text<'this> where Self: 'this;

    fn to_tui(&self, context: &RenderContext) -> Self::Output<'_> {
        Text::styled(self.text, context.theme.text_highlight_style)
    }
}

pub struct TabBrick<'a, T: FixedSelect> {
    pub tabs: &'a StatefulSelect<T>,
}

impl<'a, T: FixedSelect> ToTui for TabBrick<'a, T> {
    type Output<'this> = Tabs<'this> where Self: 'this;

    fn to_tui(&self, context: &RenderContext) -> Self::Output<'_> {
        Tabs::new(T::iter().map(|e| e.to_string()).collect())
            .select(self.tabs.selected_index())
            .highlight_style(context.theme.text_highlight_style)
    }
}

/// A list with a border and title. Each item has to be convertible to text
pub struct ListBrick<'a, T: ToTui<Output<'a> = Span<'a>>> {
    pub block: BlockBrick,
    pub list: &'a StatefulList<T>,
}

impl<'a, T: ToTui<Output<'a> = Span<'a>>> ToTui for ListBrick<'a, T> {
    type Output<'this> = List<'this> where Self: 'this;

    fn to_tui(&self, context: &RenderContext) -> Self::Output<'_> {
        let block = self.block.to_tui(context);

        // Convert each list item into text
        let items: Vec<ListItem<'_>> = self
            .list
            .items
            .iter()
            .map(|i| ListItem::new(i.to_tui(context)))
            .collect();

        List::new(items)
            .block(block)
            .highlight_style(context.theme.text_highlight_style)
            .highlight_symbol(context.theme.list_highlight_symbol)
    }
}

impl ToTui for Profile {
    type Output<'this> = Span<'this> where Self: 'this;

    fn to_tui(&self, _context: &RenderContext) -> Self::Output<'_> {
        self.name().to_owned().into()
    }
}

impl ToTui for RequestRecipe {
    type Output<'this> = Span<'this> where Self: 'this;

    fn to_tui(&self, _context: &RenderContext) -> Self::Output<'_> {
        format!("[{}] {}", self.method, self.name()).into()
    }
}

impl ToTui for Notification {
    type Output<'this> = Span<'this> where Self: 'this;

    fn to_tui(&self, _context: &RenderContext) -> Self::Output<'_> {
        format!(
            "[{}] {}",
            self.timestamp.with_timezone(&Local).format("%H:%M:%S"),
            self.message
        )
        .into()
    }
}

/// Format a timestamp in the local timezone
impl ToTui for DateTime<Utc> {
    type Output<'this> = Span<'this> where Self: 'this;

    fn to_tui(&self, _context: &RenderContext) -> Self::Output<'_> {
        self.with_timezone(&Local)
            .format("%b %e %H:%M:%S")
            .to_string()
            .into()
    }
}

impl ToTui for Duration {
    /// 'static because string is generated
    type Output<'this> = Span<'static>;

    fn to_tui(&self, _context: &RenderContext) -> Self::Output<'_> {
        let ms = self.num_milliseconds();
        if ms < 1000 {
            format!("{ms}ms").into()
        } else {
            format!("{:.2}s", ms as f64 / 1000.0).into()
        }
    }
}

impl ToTui for Option<Duration> {
    type Output<'this> = Span<'this> where Self: 'this;

    fn to_tui(&self, context: &RenderContext) -> Self::Output<'_> {
        match self {
            Some(duration) => duration.to_tui(context),
            // For incomplete requests typically
            None => "???".into(),
        }
    }
}

impl<K: Display, V: Display> ToTui for IndexMap<K, V> {
    type Output<'this> = Text<'this> where Self: 'this;

    fn to_tui(&self, _context: &RenderContext) -> Self::Output<'_> {
        self.iter()
            .map(|(key, value)| format!("{key} = {value}").into())
            .collect::<Vec<Line>>()
            .into()
    }
}

impl ToTui for HeaderMap {
    /// 'static because string is generated
    type Output<'this> = Text<'static>;

    fn to_tui(&self, _context: &RenderContext) -> Self::Output<'_> {
        self.iter()
            .map(|(key, value)| {
                format!(
                    "{key} = {}",
                    value.to_str().unwrap_or("<unrepresentable>")
                )
                .into()
            })
            .collect::<Vec<Line>>()
            .into()
    }
}

impl ToTui for anyhow::Error {
    /// 'static because string is generated
    type Output<'this> = Text<'static>;

    fn to_tui(&self, _context: &RenderContext) -> Self::Output<'_> {
        self.chain()
            .enumerate()
            .map(|(i, err)| {
                // Add indentation to parent errors
                format!("{}{err}", if i > 0 { "  " } else { "" }).into()
            })
            .collect::<Vec<Line>>()
            .into()
    }
}

/// Helper for building a layout with a fixed number of constraints
pub fn layout<const N: usize>(
    area: Rect,
    direction: Direction,
    constraints: [Constraint; N],
) -> [Rect; N] {
    Layout::default()
        .direction(direction)
        .constraints(constraints)
        .split(area)
        .as_ref()
        .try_into()
        // Should be unreachable
        .expect("Chunk length does not match constraint length")
}

/// helper function to create a centered rect using up certain percentage of the
/// available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
