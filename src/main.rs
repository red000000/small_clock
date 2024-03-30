use alarm_clock::*;

use std::sync::{Arc, Mutex};
fn main() {
    let sing_path = "蓝精灵.mp3".to_string();
    let class_table = get_class_table("class_table.json");
    let mut local_clock_table = LocalClockTable::new(class_table);
    local_clock_table.add_local_clock(sing_path);
    let playing = Arc::new(Mutex::new(true));
    LocalClockRun::run(local_clock_table, playing);
}
#[test]
fn test() {
    let class_table = panel_to_get_classes();
    for class in class_table.get_class_list() {
        println!("{}",class.get_name());
    }
}
