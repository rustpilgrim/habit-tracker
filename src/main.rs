mod types;

use std::io::{Read, Write};
use std::env;
use std::fs::File;

use types::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(args.clone());

    if args.clone().len() < 2 {
        panic!("Habit Tracker -- Error: No command specified!");
    }

    let user_data_result: Result<UserData, String> = match File::open("userdata.bin") {
        Ok(file) => {
            let mut data = Vec::new();
            let _ = file.take(u64::MAX).read_to_end(&mut data);

            match bincode::deserialize(&data) {
                Ok(decoded) => {
                    Ok(decoded)
                },
                Err(_) => {
                    Err("Unable to deserialize usedata.bin".to_string())
                },
            }
        },
        Err(e) => { 
            Err(e.to_string())
        }
    };

    let mut user_data = match user_data_result {
        Ok(data) => {
            data
        },
        Err(e) => {
            println!("{}", e);
            UserData::new()
        },
    };
    println!("{:?}", &user_data);
    
    let command = args[1].clone();
    let arg2 = args.get(2).map(|s| s.to_string());
    let arg3 = args.get(3).map(|s| s.to_string());
    let arg4 = args.get(4).map(|s| s.to_string());
    let arg5 = args.get(5).map(|s| s.to_string());

    match command.clone().as_str() {
        "help" => {
            println!("Habit Tracker Commands: ");
            println!("skip <habit> <opt date> (mark a habit as skipped, defaults to today");
            println!("complete <habit> <opt date> (mark a habit as complete, defaults to today");
            println!("fail <habit> <opt date> (mark a habit as failed, defaults to today");
            println!("increment <habit> <value> <opt date> (add value to a habit with a numerical goal, defaults to today)");
            println!("set <habit> <value> <opt date> (overwrites existing value for a habit, defaults to today)");
            println!("reset <habit> <opt date> (reset a habit node, defaults to today)");
            println!("add_habit <habit name> <desc> <goal> <opt enabled days as 1-3-5-7 etc> (adds a new habit to track)");
            println!("remove_habit <habit name> (deletes a habit and all of that habit's history)");
            println!("hide_habit <habit name> (stops showing a habit, but keeps the history saved and will not mark days as skipped)");
            println!("list <opt date> (shows a status list of all active habits at the specified date, defaults to today)");
            println!("history <habit> (shows to-date data of the specified habit, tracking % of completed days)");
            return
        },
        "reset_all" => {
            // TODO: Expand this with an extra step to prevent accidental deletion
            user_data.clear_data();
        },
        "add_habit" => {
            if let Some(habit_name) = arg2 {
                match (arg3, arg4, arg5) {
                    (Some(desc), Some(goal), Some(days)) => {
                        let new_data = HabitData::new(desc, goal.parse::<i32>().unwrap(), Some(days));
                        let result = user_data.add_habit(habit_name, new_data);
                        println!("{:?}", result);
                    },
                    (Some(desc), Some(goal), None) => {
                        let new_data = HabitData::new(desc, goal.parse::<i32>().unwrap(), None);
                        let result = user_data.add_habit(habit_name, new_data);
                        println!("{:?}", result);
                    }
                    _ => {
                        println!("Error: called add_habit without all arguments accounted for");
                    }
                }
            }
        },
        "remove_habit" => {
            if let Some(habit_name) = arg2 {
                let result = user_data.remove_habit(habit_name);
                println!("{:?}", result);
            }
        },
        "hide_habit" => {
            if let Some(habit_name) = arg2 {
                let result = user_data.hide_habit(habit_name);
                println!("{:?}", result);
            }
        },
        "complete" | "fail" | "skip" | "reset" => {
            match arg3 {
                Some(date) => {
                    let result = user_data.edit_habit_node(args.clone(), date, 0);
                    println!("{:?}", result);
                },
                None => {
                    let today = HabitData::get_current_date_id();
                    let result = user_data.edit_habit_node(args.clone(), today, 0);
                    println!("{:?}", result);
                },
            }
        },
        "increment" | "set" => {
            let mut value = 0;
            match arg3 {
                Some(v) => {
                    match v.parse::<i32>() {
                        Ok(val) => {
                            value = val;
                        },
                        Err(e) => {
                            println!("{:?}", e.to_string());
                        },
                    }
                },
                None => {
                    println!("Error: No value given for the increment command.");
                },
            }
            if value != 0 {
                match arg4 {
                    Some(date) => {
                        let result = user_data.edit_habit_node(args.clone(), date, value);
                        println!("{:?}", result);
                    },
                    None => {
                        let today = HabitData::get_current_date_id();
                        let result = user_data.edit_habit_node(args.clone(), today, value);
                        println!("{:?}", result);
                    },
                }
            }
            
        },
        "history" => {
            if let Some(habit) = arg2 {
                let _ = user_data.show_history(habit);
            }
        },
        "habit_test" => {
            let test_habit = HabitData::new("test habit!".to_string(), 1000, None);
            let _ = user_data.add_habit("test_habit".to_string(), test_habit);

            println!("Attempting to add a test habit!");
        },
        "list" => {
            match arg2 {
                Some(date) => {
                    let result = user_data.habit_list_for_day(date);
                    println!("{:?}", result);
                },
                None => {
                    let today = HabitData::get_current_date_id();
                    let result = user_data.habit_list_for_day(today);
                    println!("{:?}", result);
                },
            }
        }
        _ => {
            println!("testing hehe");
            return
        }
    }

    let command = args[1].clone();
    println!("{:?}", command);

    let serialized = bincode::serialize(&user_data).unwrap();

    let mut file = File::create("userdata.bin").unwrap();
    file.write_all(&serialized).unwrap();
}