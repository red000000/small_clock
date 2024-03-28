use std::sync::{Arc, Mutex};
use alarm_clock::*;
fn main() {
    let sing_path = "蓝精灵.mp3".to_string();
    let class_table = get_class_table("class_table.json");
    let mut local_clock_table = LocalClockTable::new(class_table);
    local_clock_table.add_local_clock(sing_path);
    let playing = Arc::new(Mutex::new(true));
    LocalClockRun::run(local_clock_table, playing);

}
