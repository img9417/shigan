use clap::Parser;
use clap::Subcommand;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use serde_json::{json, Value, to_string_pretty};
use std::fs::OpenOptions;
use chrono::{Utc, DateTime};
#[macro_use] extern crate prettytable;

/// Command line Time Tracker
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// add <TASK> to the tracker
    Add {
        /// adds a new task to the tracker
        #[arg(short, long, value_name = "TASK" )]
        task: Option<String>,
    },
    /// delete <TASK> from the tracker
    Delete {
        /// deletes a task from the tracker
        #[arg(short, long, value_name = "TASK" )]
        task: Option<String>,
    },
    /// start the tracker for <TASK>
    Start {
        #[arg(short, long, value_name = "TASK" )]
        task: Option<String>,
    },
    /// stops currently running tracker
    Stop {
        #[arg(short, long)]
        task: bool,
    },
    /// list accumulated time for <TASK>
    Log {
        /// Display the total accumulated time for TASK
        #[arg(short, long)]
        task: Option<String>,
    }
}

struct ShiganConfig {
}

impl ShiganConfig {
    fn new() -> Self {
        Self {}
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

    fn task_exists(task: &String) -> bool {
        let mut file = Self::open_file();
        let mut existing_data = String::new();
        file.read_to_string(&mut existing_data).expect("Failed to read data file");

        let mut data: Value = if existing_data.is_empty() {
            json!({ "current": {"task": "", "session": {"started": ""}}, "subjects": [] })
        } else {
            serde_json::from_str(&existing_data).expect("Failed to parse JSON data")
        };

        let subject = data["subjects"]
            .as_array_mut()
            .expect("Failed to read as an array")
            .iter_mut()
            .find(|s| s["task"].as_str().unwrap_or_default() == task);

        match subject {
            Some(_) => true,
            None => false
        }
    }

    fn add_task(&mut self, task: &String) {
        let mut file = Self::open_file();
        let mut existing_data = String::new();
        file.read_to_string(&mut existing_data).expect("Failed to read data file");

        if Self::task_exists(task) {
            println!("'{}' task already exists.", task);
            return;
        }
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
    }

    fn start_task(&mut self, task: String) {
        let mut file = Self::open_file();
        let mut existing_data = String::new();
        file.read_to_string(&mut existing_data).expect("Failed to read data file");
        let mut data: Value = if existing_data.is_empty() {
            json!({ "current": {"task": "", "session": {"started": ""}}, "subjects": [] })
        } else {
            serde_json::from_str(&existing_data).expect("Failed to parse JSON data")
        };

        let current_task = &data["current"]["task"];
        let current_task = current_task.to_string();

        if !Self::task_exists(&task) {
            println!("-- Task '{}' does not exist.", task);
            return;
        }
        if current_task.len() > 2 {
            eprintln!("-- Error - there is an ongoing task: {}", current_task);
            return;
        }

        data["current"]["task"] = json!(task);
        data["current"]["session"]["started"] = json!(Utc::now().to_rfc3339());

        file.rewind().expect("Failed to rewind data file");
        let updated_data = to_string_pretty(&data).unwrap();
        file.write_all(updated_data.as_bytes())
            .expect("Failed to write to data file");

        println!("Task '{}' starting", task);
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
        if data["current"]["task"].to_string().len() <= 2 {
            eprintln!("-- Error - there's no ongoing task.");
            return;
        }
    
        println!("Stopped tracking for the task {}", &data["current"]["task"]);

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

    fn delete_task(&mut self, task: &String) {
        let mut file = Self::open_file();
        let mut existing_data = String::new();
        file.read_to_string(&mut existing_data).expect("Failed to read data file");
        let mut data: Value = serde_json::from_str(&existing_data).expect("Failed to parse JSON data");

        let current_task = data["current"]["task"].to_owned();
        if current_task.as_str() ==  Some(task) {
            eprintln!("-- Error - cannot delete an ongoing task.");
            return;
        }
        let index = data["subjects"]
            .as_array()
            .unwrap()
            .iter()
            .position(|subject| subject["task"].as_str().unwrap_or_default() == *task);

        if let Some(position) = index {
            data["subjects"].as_array_mut().unwrap().remove(position);

            let _ = file.set_len(0);
            file.rewind().expect("Failed to rewind data file");
            let updated_data = to_string_pretty(&data).unwrap();
            file.write_all(updated_data.as_bytes())
                .expect("Failed to write to data file");
            println!("Task '{}' deleted", task);
        } else {
            println!("Task '{}' not found", task);
        }
    }

    fn log(&mut self, task: &String) {
        let mut file = Self::open_file();
        let mut existing_data = String::new();
        file.read_to_string(&mut existing_data).expect("Failed to read data file");

        let data: Value = if existing_data.is_empty() {
            json!({ "current": {"task": "", "session": {"started": ""}}, "subjects": [] })
        } else {
            serde_json::from_str(&existing_data).expect("Failed to parse JSON data")
        };
        
        let mut table = table!();
        table.add_row(row![b->"Tasks", b->"Total Minutes"]);

        match task.as_str() {
            "all" => 
            {
                let mut subjects: Vec<Value> = data["subjects"]
                .as_array()
                .unwrap()
                .iter()
                .cloned()
                .collect();
                subjects.sort_by_key(|subject| subject["durationInMinutes"].as_u64().unwrap_or_default());
                subjects.reverse();
                subjects.iter().for_each(|subject| {
                    let t = subject["task"].as_str().unwrap();
                    let d = subject["durationInMinutes"].to_string();
                    table.add_row(row![Fg->t, Fgc->d]);
                });
            },
            _ => {
                let subjects: Vec<Value> = data["subjects"].as_array().unwrap().iter().cloned().filter(|subject| subject["task"].as_str().unwrap() == *task).collect();
                
                if subjects.len() == 0 {
                    eprintln!("-- Error - No task found");
                } else {
                    let t = subjects[0]["task"].as_str().unwrap();
                    let d = subjects[0]["durationInMinutes"].to_string();
                    table.add_row(row![Fg->t, Fgc->d]);
                }
            }
        }
        table.printstd();
    }
}

fn main() {
    let cli = Cli::parse();
    let mut shigan= ShiganConfig::new();
    shigan.init();

    match &cli.command {
        Some(Commands::Add { task }) => {
            match task {
                Some(t) => shigan.add_task(&(t.to_lowercase())),
                None => println!("None")
            }
        }
        Some(Commands::Delete { task }) => {
            match task {
                Some(t) => shigan.delete_task(&(t.to_lowercase())),
                None => println!("None")
            }
        }
        Some(Commands::Start { task }) => {
            match task {
                Some(t) => shigan.start_task(t.to_lowercase()),
                None => println!("None")
            }
        }
        Some(Commands::Log { task }) => {
            match task {
                Some(t) => shigan.log(&(t.to_lowercase())),
                None => shigan.log(&"all".to_string().to_lowercase())
            }
        }
        Some(Commands::Stop { task: _ }) => {
            shigan.end_task();
        }
        None => {}
    }
}