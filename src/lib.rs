use chrono::{Datelike, Local, Timelike, Weekday};
use rodio::Sink;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
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
    pub fn new_from(
        class_name: String,
        class_teacher: String,
        class_hour: u32,
        class_minute: u32,
        class_weekday: u32,
    ) -> Self {
        Class {
            class_name,
            class_teacher,
            class_hour,
            class_minute,
            class_weekday,
        }
    }
    pub fn set(&mut self, class_name: String, class_teacher: String) {
        self.class_name = class_name;
        self.class_teacher = class_teacher;
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
    pub fn new() -> Self {
        ClassTable {
            class_list: Vec::new(),
        }
    }
    pub fn get_class_list(&self) -> &Vec<Class> {
        &self.class_list
    }
    fn add_class(&mut self, class: Class) {
        self.class_list.push(class);
    }
}
impl Clone for ClassTable {
    fn clone(&self) -> Self {
        ClassTable {
            class_list: self.class_list.clone(),
        }
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
        let file = fs::File::open(&self.sing_path).expect("Failed to open audio file");
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
    local_clock_table: Vec<LocalClock>,
    class_table: ClassTable,
}
impl LocalClockTable {
    pub fn new(class_table: ClassTable) -> Self {
        LocalClockTable {
            local_clock_table: Vec::new(),
            class_table: class_table,
        }
    }
    pub fn add_local_clocks(&mut self, sing_path: String) {
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
pub fn json_remove_and_write(path: &str, class_table: &ClassTable) {
    //文件重置后写入，可用
    match fs::remove_file(path) {
        Ok(_) => {}
        Err(_) => {
            //文件不存在，无需删除
        }
    }

    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .unwrap();
    let file_wirter = std::io::BufWriter::new(file);
    serde_json::to_writer(file_wirter, class_table).unwrap();
}
//待修改，gui界面
pub fn panel_to_get_classes() -> ClassTable {
    use fltk::{app, button::Button, enums::Color, group::Group, input::Input, prelude::*, window};

    let class_table = Rc::new(RefCell::new(ClassTable::new()));
    let class_table_clone = Rc::clone(&class_table);
    let app = app::App::default();
    // 创建窗口
    let mut wind = window::Window::new(100, 100, 400, 300, "课程信息输入");
    wind.set_color(Color::Black);

    // 创建面板
    let mut panel = Group::new(10, 10, 380, 280, "");
    panel.set_color(Color::Black);

    // 课程名称输入
    let class_name_input = Rc::new(RefCell::new(Input::new(80, 30, 300, 30, "课程名称")));
    class_name_input
        .as_ref()
        .borrow_mut()
        .set_color(Color::White);
    class_name_input
        .as_ref()
        .borrow_mut()
        .set_label_color(Color::White);
    let class_name_input_clone = Rc::clone(&class_name_input);
    // 教师输入
    let teacher_input = Rc::new(RefCell::new(Input::new(80, 70, 300, 30, "教师")));
    teacher_input.as_ref().borrow_mut().set_color(Color::White);
    teacher_input
        .as_ref()
        .borrow_mut()
        .set_label_color(Color::White);
    let teacher_input_clone = Rc::clone(&teacher_input);

    // 课时输入
    let hour_input = Rc::new(RefCell::new(Input::new(80, 110, 100, 30, "课时")));
    hour_input.as_ref().borrow_mut().set_color(Color::White);
    hour_input
        .as_ref()
        .borrow_mut()
        .set_label_color(Color::White);
    let hour_input_clone = Rc::clone(&hour_input);

    // 分钟输入
    let minute_input = Rc::new(RefCell::new(Input::new(220, 110, 100, 30, "分钟")));
    minute_input.as_ref().borrow_mut().set_color(Color::White);
    minute_input
        .as_ref()
        .borrow_mut()
        .set_label_color(Color::White);
    let minute_input_clone = Rc::clone(&minute_input);

    // 星期需要进行处理
    let weekday_input = Rc::new(RefCell::new(Input::new(80, 150, 100, 30, "星期")));
    weekday_input.as_ref().borrow_mut().set_color(Color::White);
    weekday_input
        .as_ref()
        .borrow_mut()
        .set_label_color(Color::White);
    let weekday_input_clone = Rc::clone(&weekday_input);

    // 提交按钮
    let mut submit_button = Button::new(160, 200, 100, 50, "提交单个课程");
    submit_button.set_color(Color::White);

    let mut submit_all_button = Button::new(270, 200, 100, 50, "提交课表完成");
    submit_all_button.set_color(Color::White);

    // 显示窗口
    wind.end();
    wind.show();

    // 事件处理,待修改
    submit_button.set_callback(move |_| {
        let class_name = class_name_input_clone.as_ref().borrow().value();
        let teacher = teacher_input_clone.as_ref().borrow().value();
        let hour = hour_input_clone.as_ref().borrow().value();
        let minute = minute_input_clone.as_ref().borrow().value();
        let weekday = weekday_input_clone.as_ref().borrow().value();
        //判断是否为空
        if class_name.is_empty()
            || teacher.is_empty()
            || hour.is_empty()
            || minute.is_empty()
            || weekday.is_empty()
        {
            fltk::dialog::alert(200, 200, "不允许有空的信息");
            return;
        }
        let hour = hour.parse::<u32>().unwrap_or(0);
        let minute = minute.parse::<u32>().unwrap_or(0);
        let weekday = weekday.parse::<u32>().unwrap_or(0);
        class_table_clone
            .as_ref()
            .borrow_mut()
            .add_class(Class::new_from(class_name, teacher, hour, minute, weekday));
        //清除输入
        class_name_input_clone.as_ref().borrow_mut().set_value("");
        teacher_input_clone.as_ref().borrow_mut().set_value("");
        hour_input_clone.as_ref().borrow_mut().set_value("");
        minute_input_clone.as_ref().borrow_mut().set_value("");
        weekday_input_clone.as_ref().borrow_mut().set_value("");
    });
    submit_all_button.set_callback(move |_| {
        class_name_input.as_ref().borrow_mut().set_value("");
        teacher_input.as_ref().borrow_mut().set_value("");
        hour_input.as_ref().borrow_mut().set_value("");
        minute_input.as_ref().borrow_mut().set_value("");
        weekday_input.as_ref().borrow_mut().set_value("");
        app.quit();
    });
    app.run().unwrap();
    if app::event() == fltk::enums::Event::Released {
        class_table.as_ref().borrow_mut().clone()
    } else {
        ClassTable::new()
    }
}
