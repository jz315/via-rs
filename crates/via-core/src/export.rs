use crate::error::Result;
use crate::model::Board;

pub trait Exporter {
    type Output;

    fn export_board(&self, board: &Board) -> Result<Self::Output>;
}
