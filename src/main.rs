use alarm_clock::{get_class_table, LocalClock};


fn main() {
    let clock=LocalClock::new();
    let class_table=get_class_table("class_table.json");
    for class in class_table.get_class_list(){
        clock.work(class);
    }
}