use std::cell::RefCell;

use ratatui::style::{Color, Stylize};
use ratatui::text::Span;
use ratatui::widgets::canvas::Line;
use std::fmt::Debug;

pub mod austrailia;
pub trait Region: Clone + Copy + Debug + PartialEq {
    /// Returns coordinates to where the labels should start
    /// appearing
    fn coordinates(&self) -> (f64, f64);

    /// Returns the first area in the region
    fn default() -> Self;
}

#[derive(Clone, Debug)]
pub struct TouristSite<R: Region> {
    name: String,
    id: String,
    region: R,
    completed: bool,
}

impl<'a, R: Region> TouristSite<R> {
    const CROSS_LENGTH: f64 = 5.0;
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }
    pub fn region(&self) -> R {
        self.region.clone()
    }
    pub fn render(self, ctx: &mut ratatui::widgets::canvas::Context<'a>, offset: f64) {
        let (x, y) = self.region.coordinates();
        let id: String = self.id;
        ctx.layer();
        match self.completed {
            true => ctx.print(x + offset, y, id),
            _ => ctx.print(x + offset, y, id),
        };
        if self.completed {
            // Ugly hack to get around the borrow checker.
            //
            // The problem here is that id cannot have the same lifetime as the
            // ctx object, this is fine but the lifetimes introduced in ratatui
            // causes some issues here.
            // Instead of dealing with these we draw a "line"
            // behind the letter in question.
            ctx.draw(&Line {
                x1: x + offset,
                y1: y,
                x2: x + offset,
                y2: y,
                color: Color::Yellow,
            });
        }
        else{
            ctx.draw(&Line {
                x1: x + offset,
                y1: y,
                x2: x + offset,
                y2: y,
                color: Color::DarkGray,
            });

        }
    }
    pub fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            id: self.id.clone(),
            region: self.region.clone(),
            completed: self.completed.clone(),
        }
    }
    /// Draws the site, if the players has completed it
    /// it will be golden if not it will be gray
    pub fn get<'b>(&'a self) -> ((f64, f64), String, bool) {
        (self.region.coordinates(), self.id.clone(), self.completed)
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
