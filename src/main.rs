use clap::{Parser, Subcommand};
use std::fs::{create_dir_all, OpenOptions, File};
use std::io::prelude::*;
use serde_json::{json, Value, to_string_pretty};
use chrono::{Utc, DateTime};
use std::path::PathBuf;
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
    /// Add a task to the tracker
    Add {
        #[arg(short, long, value_name = "TASK" )]
        task: Option<String>,
    },
    /// Deletes a task from the tracker
    Delete {
        #[arg(short, long, value_name = "TASK" )]
        task: Option<String>,
    },
    /// Starts the tracker for <TASK>
    Start {
        #[arg(short, long, value_name = "TASK" )]
        task: Option<String>,
    },
    /// Stops currently running tracker
    Stop {
        #[arg(short, long)]
        task: bool,
    },
    /// List accumulated time for the task or all (default="all")
    Log {
        #[arg(short, long)]
        task: Option<String>,
    }
}

struct ShiganConfig {
    shigan_dir: PathBuf
}

impl ShiganConfig {
    fn new() -> Self {
        let home_dir = dirs::home_dir().expect("Failed to get home directory");
        let shigan_dir = home_dir.join(".shigan");

        Self {
            shigan_dir,
        }
    }

    fn init(&mut self){
        if !self.shigan_dir.exists() {
            create_dir_all(&self.shigan_dir).expect("Failed to create tracker directory");
            println!("Shigan directory created: {:?}", self.shigan_dir);
        }
    }

    fn open_file(&mut self) -> File {
        let data_file_path = self.shigan_dir.join("data.json");
        OpenOptions::new() 
        .read(true)
        .write(true)
        .create(true)
        .open(&data_file_path)
        .expect("Failed to open data file")
    }

    fn read_data(&mut self, file: &mut File) -> Value {
        let mut existing_data = String::new();
        file.read_to_string(&mut existing_data).expect("Failed to read data file");
    
        let data: Value = if existing_data.is_empty() {
            json!({ "current": {"task": "", "session": {"started": ""}}, "subjects": [] })
        } else {
            serde_json::from_str(&existing_data).expect("Failed to parse JSON data")
        };

        return data;
    }

    fn write_data(&mut self, data: &Value, file: &mut File) {
        file.rewind().expect("Failed to rewind data file");
        let updated_data = to_string_pretty(&data).unwrap();
        file.write_all(updated_data.as_bytes())
            .expect("Failed to write to data file");
    }

    fn task_exists(&mut self, task: &String) -> bool {
        let mut file = self.open_file();
        let mut data = self.read_data(&mut file);

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
        let mut file = self.open_file();
        let mut data = self.read_data(&mut file);

        if self.task_exists(task) {
            println!("'{}' task already exists.", task);
            return;
        }

        data["subjects"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "task": task,
                "durationInMinutes": 0,
                "sessions": []
            }));
        
        self.write_data(&data, &mut file);
    }

    fn start_task(&mut self, task: String) {
        let mut file = self.open_file();
        let mut data = self.read_data(&mut file);

        let current_task = &data["current"]["task"];
        let current_task = current_task.to_string();

        if !self.task_exists(&task) {
            println!("-- Task '{}' does not exist.", task);
            return;
        }
        if current_task.len() > 2 {
            eprintln!("@@ Error - there is an ongoing task: {}", current_task);
            return;
        }

        data["current"]["task"] = json!(task);
        data["current"]["session"]["started"] = json!(Utc::now().to_rfc3339());

        self.write_data(&data, &mut file);

        println!("Task '{}' starting", task);
    }

    fn end_task(&mut self) {
        let mut file = self.open_file();
        let mut data = self.read_data(&mut file);

        let current_session_start: DateTime<Utc> = DateTime::parse_from_rfc3339(
            data["current"]["session"]["started"].as_str().unwrap_or_default(),
        )
        .unwrap_or_else(|_| Utc::now().into()).into();

        let current_session_end = Utc::now();
        let current_session_duration = current_session_end - current_session_start;
        let current_task = data["current"]["task"].to_owned();
        if data["current"]["task"].to_string().len() <= 2 {
            eprintln!("@@ Error - there's no ongoing task.");
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

        self.write_data(&data, &mut file);
    }

    fn delete_task(&mut self, task: &String) {
        let mut file = self.open_file();
        let mut data = self.read_data(&mut file);

        let current_task = data["current"]["task"].to_owned();
        if current_task.as_str() ==  Some(task) {
            eprintln!("@@ Error - cannot delete an ongoing task.");
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
            self.write_data(&data, &mut file);
            println!("Task '{}' deleted", task);
        } else {
            println!("Task '{}' not found", task);
        }
    }

    fn log(&mut self, task: &String) {
        let mut file = self.open_file();
        let data = self.read_data(&mut file);
        
        let mut table = table!();
        table.add_row(row![b->"Tasks", b->"Time Accumulated"]);

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
                    let d = subject["durationInMinutes"].to_string().parse::<u32>().unwrap();
                    let h = d / 60;
                    let d = d % 60;
                    let hd = format!("{:>3}h {:>3}m", h, d);
                    table.add_row(row![Fg->t, Fgc->hd]);
                });
            },
            _ => {
                let subjects: Vec<Value> = data["subjects"].as_array().unwrap().iter().cloned().filter(|subject| subject["task"].as_str().unwrap() == *task).collect();
                
                if subjects.len() == 0 {
                    eprintln!("@@ Error - No task found");
                } else {
                    let t = subjects[0]["task"].as_str().unwrap();
                    let d = subjects[0]["durationInMinutes"].to_string().parse::<u32>().unwrap();
                    let h = d / 60;
                    let d = d % 60;
                    let hd = format!("{:>3}h {:>3}m", h, d);

                    table.add_row(row![Fg->t, Fgc->hd]);
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