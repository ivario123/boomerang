use crossterm::style::Stylize;

pub mod austrailia;
pub trait Region {
    /// Returns coordinates to where the labels should start
    /// appearing
    fn coordinates() -> (f64, f64);
}

#[derive(Clone)]
pub struct TouristSite<R: Region> {
    name: String,
    id: String,
    region: R,
    completed: bool,
}

impl<R: Region> TouristSite<R> {
    /// Draws the site, if the players has completed it
    /// it will be golden if not it will be gray
    pub fn get(&self) -> crossterm::style::StyledContent<&str> {
        match self.completed {
            true => self.id.as_str().clone().yellow(),
            false => self.id.as_str().clone().grey(),
        }
    }
    /// Creates a new Site at the given
    /// site
    pub fn new(name: String, id: String, region: R) -> Self {
        Self {
            name,
            id,
            region,
            completed: false,
        }
    }
}
