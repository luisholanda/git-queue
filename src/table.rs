use prettytable::{Attr, Cell, Row};

pub fn new(columns: impl Iterator<Item = &'static str>) -> prettytable::Table {
    let mut table = prettytable::Table::new();
    table.set_titles(Row::new(
        columns
            .map(|c| Cell::from(&c).with_style(Attr::Underline(true)).with_style(Attr::Bold))
            .collect(),
    ));
    let mut format = prettytable::format::TableFormat::new();
    format.column_separator(' ');
    table.set_format(format);
    table
}
