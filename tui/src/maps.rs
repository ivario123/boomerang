use ratatui::style::Color;
use ratatui::widgets::canvas::Shape;

use self::sites::{Region, TouristSite};

mod australia;
pub mod sites;

/// Exports all objects relevant to the boomerang australia map.
pub mod boomerang_australia {
    pub use super::australia::Map;
    pub use super::sites::australia::Region;
}

pub trait Map: Shape {
    type REGION: Region;
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

    fn render(
        &self,
        ctx: &mut ratatui::widgets::canvas::Context<'_>,
    ) -> Vec<TouristSite<Self::REGION>>;
}
