mod spreadsheet;
use crate::spreadsheet::{CellFormula, Spreadsheet, ref_};

fn main() {
    let mut sheet = Spreadsheet::new();
    sheet.set(0, 0, ref_(0, 0));
}
