use chrono::Weekday;
use rodio::{Sink, Source};
use std::fs::File;
use chrono::{Datelike, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use std::{fs, io::BufReader};
#[derive(Serialize, Deserialize)]
pub struct Class {
    class_name: String,
    class_teacher: String,
    class_hour: u32,
    class_minute: u32,
    class_weekday: u32,
}
#[derive(Serialize, Deserialize)]
pub struct ClassTable {
    class_list: Vec<Class>,
}
impl ClassTable {
    pub fn get_class_list(&self) -> &Vec<Class> {
        &self.class_list
    }
}
pub struct LocalClock {
    local_hour: u32,
    local_minute: u32,
    weekday: Weekday,
}
impl LocalClock {
    pub fn new() -> Self {
        // 获取当前本地周几
        let local_time = Local::now();
        let local_hour = local_time.hour();
        let local_minute = local_time.minute();
        let weekday = local_time.weekday();
        LocalClock {
            local_hour,
            local_minute,
            weekday,
        }
    }

    pub fn work(&self, class: &Class) {
        loop {
            let hour_check = self.local_hour == class.class_hour;
            let minute_check = self.local_minute == class.class_minute;
            let weekday_check = weekday_to_u32(&self.weekday) == class.class_weekday;
            if hour_check && minute_check && weekday_check {
                play_sound("蓝精灵.mp3");
                break;
            }
        }
    }
}
pub fn get_class_table(path: &str) -> ClassTable {
    let file = fs::File::open(path).expect("Failed to open file");
    let reader = BufReader::new(file);
    let class_table: ClassTable = serde_json::from_reader(reader).expect("Failed to parse JSON");
    class_table
}
fn weekday_to_u32(weekday: &Weekday) -> u32 {
    match weekday {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}
fn play_sound(path: &str) {
    // 打开音频文件
    let file = File::open(path).expect("Failed to open audio file");
    let source = rodio::Decoder::new(BufReader::new(file)).expect("Failed to decode audio file");

    // 创建音频播放器
    let (_stream, handle) =
        rodio::OutputStream::try_default().expect("Failed to create output stream");
    let sink = Sink::try_new(&handle).expect("Failed to create sink");

    let duration = source.total_duration();
    println!("{:?}", duration);
    // 将音频添加到播放器中
    sink.append(source);

    // 播放音频
    sink.play();

    // 等待音频播放完毕
    std::thread::sleep(Duration::from_secs(30));

    // 关闭播放器
}
