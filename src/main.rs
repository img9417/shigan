use clap::Parser;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use serde_json::{json, Value, to_string_pretty};
use std::fs::OpenOptions;
use chrono::{Utc, DateTime};

#[derive(Parser)]
#[command(version, about = "Command line Pomodoro Timer", long_about = None)]
struct Args {
    #[arg(short, long, help="add <TASK> to the tracker", value_name = "TASK" )]
    add: Option<String>,

    #[arg(short, long, help="delete <TASK> from the tracker", value_name = "TASK" )]
    delete: Option<String>,

    #[arg(long, help="start the tracker for <TASK>", value_name = "TASK" )]
    start: Option<String>,

    #[arg(long, help="stops currently running tracker")]
    stop: bool,

    #[arg(short, long, help="list accumulated time for <TASK>", value_name = "TASK")]
    log: Option<String>,
}

struct ShiganConfig {
    current_task: Option<String>,
    stop: bool,
}

impl ShiganConfig {
    fn new() -> Self {
        Self {
            current_task: None,
            stop: false
        }
    }

    fn init(&mut self){
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let shigan_dir = home_dir.join(".shigan");

        if !shigan_dir.exists() {
            create_dir_all(&shigan_dir).expect("Failed to create tracker directory");
            println!("Shigan director created: {:?}", shigan_dir);
        }
    }

    fn open_file() -> File {
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let shigan_dir = home_dir.join(".shigan");
        let data_file_path = shigan_dir.join("data.json");
        OpenOptions::new() 
            .read(true)
            .write(true)
            .create(true)
            .open(&data_file_path)
            .expect("Failed to open data file")
    }

    fn add_task(&mut self, task: &String) {
        let mut file = Self::open_file();
        let mut existing_data = String::new();
        file.read_to_string(&mut existing_data).expect("Failed to read data file");

        let mut data: Value = if existing_data.is_empty() {
            json!({ "current": {"task": "", "session": {"started": ""}}, "subjects": [] })
        } else {
            serde_json::from_str(&existing_data).expect("Failed to parse JSON data")
        };
        
        data["subjects"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "task": task,
                "durationInMinutes": 0,
                "sessions": []
            }));
        
        file.rewind().expect("Failed to rewind data file");
        let updated_data = to_string_pretty(&data).unwrap();
        file.write_all(updated_data.as_bytes())
            .expect("Failed to write to data file");

        // println!("Task '{}' added to data file: {:?}", task, data_file_path);
    }

    fn start_task(&mut self) {
        let mut file = Self::open_file();
        let mut existing_data = String::new();
        file.read_to_string(&mut existing_data).expect("Failed to read data file");
        let mut data: Value = serde_json::from_str(&existing_data).expect("Failed to parse JSON data");

        let task = &self.current_task;
        data["current"]["task"] = json!(task.clone().unwrap());
        data["current"]["session"]["started"] = json!(Utc::now().to_rfc3339());

        file.rewind().expect("Failed to rewind data file");
        let updated_data = to_string_pretty(&data).unwrap();
        file.write_all(updated_data.as_bytes())
            .expect("Failed to write to data file");

        println!("Task '{}' starting", task.clone().unwrap());
    }

    fn end_task(&mut self) {
        let mut file = Self::open_file();
        let mut existing_data = String::new();
        file.read_to_string(&mut existing_data).expect("Failed to read data file");
        let mut data: Value = serde_json::from_str(&existing_data).expect("Failed to parse JSON data");

        let current_session_start: DateTime<Utc> = DateTime::parse_from_rfc3339(
            data["current"]["session"]["started"].as_str().unwrap_or_default(),
        )
        .unwrap_or_else(|_| Utc::now().into()).into();
        let current_session_end = Utc::now();
        let current_session_duration = current_session_end - current_session_start;
        let current_task = data["current"]["task"].to_owned();
        let subject = data["subjects"]
            .as_array_mut()
            .expect("Failed to read as an array")
            .iter_mut()
            .find(|s| s["task"].as_str().unwrap_or_default() == current_task)
            .expect("Task not found in subjects");

        subject["sessions"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "started": current_session_start.to_rfc3339(),
            "ended": current_session_end.to_rfc3339(),
            "duration": format!("{}h {}m {}s", current_session_duration.num_hours(), current_session_duration.num_minutes() % 60, current_session_duration.num_seconds() % 60)
        }));

        subject["durationInMinutes"] = json!(subject["durationInMinutes"].as_u64().unwrap_or_default()
        + (current_session_duration.num_seconds() / 60) as u64);

        data["current"] = json!({
            "task": "",
            "session": {}
        });
        file.rewind().expect("Failed to rewind data file");
        let updated_data = to_string_pretty(&data).unwrap();
        file.write_all(updated_data.as_bytes())
            .expect("Failed to write to data file");
    }
}

fn main() {
    let cli = Args::parse();
    let mut shigan= ShiganConfig::new();
    shigan.init();

    if let Some(task) = cli.add {
        match task {
            _ => {
                shigan.add_task(&(task.to_lowercase()));
            }
        }
    }

    if let Some(task) = cli.delete {
        match task {
            _ => println!("'{task}' deleted"),
        }
    }

    if let Some(task) = cli.start {
        match task {
            _ => {
                shigan.current_task = Some(task.to_lowercase());
                shigan.start_task();
            }
        };
    }

    if cli.stop {
        shigan.end_task();
    }

    if let Some(task) = cli.log {
        match task.as_str() {
            "all" => println!("log all"),
            _ => println!("log individual, {}", task)
        }
    }
}