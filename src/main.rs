//隐藏cmd窗口,且不会在命令提示符中显示任何输出
#![windows_subsystem = "windows"]

use alarm_clock::*;
use std::fs;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
fn main() {
    if fs::metadata("config").is_err() {
        //如果config文件不存在,则创建config文件
        let mut select_mode = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open("config")
            .unwrap();
        select_mode.write("gui_mode:true\n".as_bytes()).unwrap();
    }

    let config = fs::read_to_string("config").unwrap();
    for line in config.lines() {
        //目前只判断是否使用gui输入课表,当然音频路径还是得用gui的
        let mode_type = line.split(":").collect::<Vec<&str>>();
        if mode_type[0] == "gui_mode" {
            if mode_type[1] == "true" {
                panel();
                let mut config_file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("config")
                    .unwrap();
                config_file.write("gui_mode:false\n".as_bytes()).unwrap();
            }else{
                json_run();
            }
        }
    }
}

//json_run();

//两个版本都发布,面向不同喜好的用户
pub fn json_run() {
    //获取课表
    let class_table = get_class_table("class_table.json");
    //设置提醒声音路径
    let sing_path = panel_to_get_music();
    println!("{}",sing_path);
    //初始化“是否默认音频播放器正在被使用”
    let playing = Arc::new(Mutex::new(true));
    //写入课表到本地
    json_remove_and_write("class_table.json", &class_table);
    //获取本地课表
    let mut local_clock_table = LocalClockTable::new(class_table);
    //设置闹钟音频路径
    local_clock_table.add_local_clocks(sing_path);
    //运行闹钟
    LocalClockRun::run(local_clock_table, playing);
}
pub fn panel() {
    let playing = Arc::new(Mutex::new(true));
    //启动gui
    let class_table = panel_to_get_classes();
    let sing_path = panel_to_get_music();
    println!("{}", sing_path);
    json_remove_and_write("class_table.json", &class_table);
    let mut local_clock_table = LocalClockTable::new(class_table);
    local_clock_table.add_local_clocks(sing_path);
    LocalClockRun::run(local_clock_table, playing);
}
