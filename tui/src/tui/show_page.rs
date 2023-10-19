use super::TuiPage;

pub trait ShowPage: TuiPage {
    fn eq(&self, other: &Self) -> bool;
}
