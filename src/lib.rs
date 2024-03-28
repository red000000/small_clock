use chrono::Weekday;
use chrono::{Datelike, Local, Timelike};
use rodio::Sink;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
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
impl Class {
    pub fn new() -> Self {
        Class {
            class_name: String::new(),
            class_teacher: String::new(),
            class_hour: 0,
            class_minute: 0,
            class_weekday: 0,
        }
    }
    pub fn get_name(&self) -> &str {
        &self.class_name
    }
    pub fn get_teacher(&self) -> &str {
        &self.class_teacher
    }
}
impl Clone for Class {
    fn clone(&self) -> Self {
        Class {
            class_name: self.class_name.clone(),
            class_teacher: self.class_teacher.clone(),
            class_hour: self.class_hour,
            class_minute: self.class_minute,
            class_weekday: self.class_weekday,
        }
    }
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
    sing_path: String,
    class: Class,
}
impl LocalClock {
    pub fn new(sing_path: String) -> Self {
        // 获取当前本地周几
        let local_time = Local::now();
        let local_hour = local_time.hour();
        let local_minute = local_time.minute();
        let weekday = local_time.weekday();
        let class = Class::new();
        LocalClock {
            local_hour,
            local_minute,
            weekday,
            sing_path,
            class,
        }
    }
    pub fn add_local_clock(&mut self, class: Class) {
        self.class = class;
    }
    pub fn update(&mut self) {
        let local_time = Local::now();
        self.local_hour = local_time.hour();
        self.local_minute = local_time.minute();
        self.weekday = local_time.weekday()
    }
    pub fn play_sound(&self) {
        // 打开音频文件
        let file = File::open(&self.sing_path).expect("Failed to open audio file");
        let source =
            rodio::Decoder::new(BufReader::new(file)).expect("Failed to decode audio file");

        // 创建音频播放器
        let (_stream, handle) =
            rodio::OutputStream::try_default().expect("Failed to create output stream");
        let sink = Sink::try_new(&handle).expect("Failed to create sink");

        // 将音频添加到播放器中
        sink.append(source);

        // 播放音频
        sink.play();

        // 关闭播放器
        sink.sleep_until_end();
    }
    pub fn work(&mut self, playing: &Arc<Mutex<bool>>) {
        loop {
            self.update();
            let mut playing_lock = playing.lock().unwrap();
            //println!("获取扬声器锁{}", *playing_lock);
            if *playing_lock {
                //检查是否有进程占用默认扬声器,若没有，修改playing为false
                let handle = rodio::OutputStream::try_default();
                if handle.is_ok() {
                    *playing_lock = false;
                    //println!("尝试获取扬声器成功");
                } else {
                    //println!("扬声器被占用,{}", self.class.get_name());
                }
            } else {
                let hour_check = self.local_hour == self.class.class_hour;
                let minute_check = self.local_minute == self.class.class_minute;
                let weekday_check = weekday_to_u32(&self.weekday) == self.class.class_weekday;
                //println!("{}检查时间,local_hour:{},local_minute:{}",self.class.get_name(),self.local_hour,self.local_minute); debug
                if hour_check && minute_check && weekday_check {
                    self.play_sound();
                    //println!("运行结束,{}", self.class.get_name());
                    *playing_lock = false;
                    break;
                } else if self.local_hour > self.class.class_hour
                    || self.local_minute > self.class.class_minute
                {
                    //↑此判断需要修改 2024.3.28，逻辑修改无误后请删除本注释
                    
                    // println!("此线程等待播放音乐已超时，退出,课程名为:{},课程老师为:{}",self.class.get_name(),self.class.get_teacher()); debug
                    break;
                }
            }
            //停顿一秒,检查不用太快，若课表比较庞大则需要调低一些，数量待测试
            thread::sleep(Duration::from_secs(1));
        }
        //println!("线程结束,课程名为:{},课程老师为:{}",self.class.get_name(),self.class.get_teacher());
    }
}
pub struct LocalClockTable {
    pub local_clock_table: Vec<LocalClock>,
    pub class_table: ClassTable,
}
impl LocalClockTable {
    pub fn new(class_table: ClassTable) -> Self {
        LocalClockTable {
            local_clock_table: Vec::new(),
            class_table: class_table,
        }
    }
    pub fn add_local_clock(&mut self, sing_path: String) {
        //几个课表对应几个闹钟
        for class in self.class_table.get_class_list() {
            let mut local_clock = LocalClock::new(sing_path.clone());
            local_clock.add_local_clock(class.clone());
            self.local_clock_table.push(local_clock);
        }
    }
}
pub struct LocalClockRun {}

impl LocalClockRun {
    pub fn run(local_clock_table: LocalClockTable, playing: Arc<Mutex<bool>>) {
        // 创建一个空的 Vec 用于存放所有线程的 JoinHandle
        let mut handles = Vec::new();
        // 启动所有线程并将它们的 JoinHandle 放入 Vec 中
        for mut local_clock in local_clock_table.local_clock_table {
            let playing_clone = Arc::clone(&playing);
            let handle = thread::spawn(move || {
                local_clock.work(&playing_clone);
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
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
