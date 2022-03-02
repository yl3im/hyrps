use comfy_table::{presets::UTF8_BORDERS_ONLY, Table};

use super::Codeplug;

pub trait DisplayTabular {
    fn get_heading() -> Vec<String>;
    fn get_row(&self, codeplug: &Codeplug) -> Vec<String>;

    fn print_table(objs: &[Self], codeplug: &Codeplug)
    where
        Self: Sized,
    {
        let mut table = Table::new();

        table.load_preset(UTF8_BORDERS_ONLY);

        let header = Self::get_heading();
        table.set_header(&header);

        for obj in objs.iter() {
            let row = obj.get_row(codeplug);
            table.add_row(&row);
        }

        println!("{table}");
    }
}
