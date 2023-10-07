use std::borrow::BorrowMut;

use crossterm::style::Stylize;
use ratatui::style::Color;
use ratatui::widgets::canvas::Shape;

pub mod boomerang_australia;
pub mod sites;

pub trait Map: Shape {
    const WIDTH: usize;
    const HEIGHT: usize;

    /// Creates a default initiation of the map
    fn default() -> Self;

    /// Returns all of the points that will be drawn in the UI
    fn map(&self) -> Vec<(usize, usize)>;

    /// Sets the color of the map
    fn set_color(&mut self, color: Color);

    /// Returns the current color of the map
    fn get_color(&self) -> Color;

    fn render(&self, ctx: &mut ratatui::widgets::canvas::Context<'_>);
}
