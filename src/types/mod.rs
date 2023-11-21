use std::{collections::HashMap, num::ParseIntError};
use chrono::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum NodeStatus {
    IDLE,
    SKIPPED,
    PARTIAL,
    FAILED,
    COMPLETE
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HabitID {
    pub day: u32,
    pub month: u32,
    pub year: u32,
}

impl HabitID {
    pub fn to_string(&self) -> String {
        let id = format!("{}-{}-{}",
            self.month.to_string(),
            self.day.to_string(),
            self.year.to_string());

        return id;
    }

    pub fn from_string(val: String) -> Self {
        let parts = val.split("-").collect::<Vec<&str>>();

        let id = HabitID {
            month: parts[0].parse::<u32>().unwrap(),
            day: parts[1].parse::<u32>().unwrap(),
            year: parts[2].parse::<u32>().unwrap(),
        };

        return id;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HabitNode {
    value: i32,
    goal: i32,  //derived from HabitData.goal
    status: NodeStatus
}

impl HabitNode {
    fn idle_node(&mut self) {
        self.status = NodeStatus::IDLE;
    }

    fn skip_node(&mut self) {
        self.status = NodeStatus::SKIPPED;
    }

    fn fail_node(&mut self) {
        self.status = NodeStatus::FAILED;
    }

    fn complete_node(&mut self) {
        self.status = NodeStatus::COMPLETE;
        self.value = self.goal;
    }

    fn calculate_status(&mut self) -> NodeStatus {
        if self.value < self.goal {
            self.status = NodeStatus::PARTIAL;
            return NodeStatus::PARTIAL;
        } else {
            self.status = NodeStatus::COMPLETE;
            return NodeStatus::COMPLETE;
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HabitData {
    start_year: u32,
    start_month: u32,
    start_day: u32,
    enabled_days: Option<Vec<u32>>,
    description: String,
    goal: i32, // ex: habit is walk 5000 steps per day, size would be 5000
    nodes: HashMap<String, HabitNode>, // key is month-day-year -> oct 4 2023 = 10-4-2023
    metrics: HashMap<NodeStatus, i32>, // number of nodes in the habit with each status
    active: bool,
}

impl HabitData {
    pub fn new(desc: String, goal: i32, days: Option<String>) -> Self {
        let today = chrono::Local::now();
        let mut fresh_metrics: HashMap<NodeStatus, i32> = HashMap::new();
        let keys = vec![NodeStatus::IDLE, NodeStatus::FAILED, NodeStatus::PARTIAL, NodeStatus::SKIPPED, NodeStatus::COMPLETE];

        for key in keys {
            fresh_metrics.insert(key, 0);
        }
        
        match days {
            Some(d) => {
                match HabitData::validate_allowed_days(d) {
                    Ok(p) => {
                        return HabitData {
                            start_year: today.year() as u32,
                            start_month: today.month(),
                            start_day: today.day(),
                            enabled_days: Some(p),
                            description: desc,
                            goal: goal,
                            nodes: HashMap::new(),
                            metrics: fresh_metrics,
                            active: true,
                        }
                    },
                    Err(_) => {
                        println!("Error: Invalid enabled_days argument, so the habit has been created with all days enabled.");
                        println!("     Delete the habit and try again, or use the edit_days command to enter a valid string (ex: '1-3-5-7') to set the days properly.");
                        return HabitData {
                            start_year: today.year() as u32,
                            start_month: today.month(),
                            start_day: today.day(),
                            enabled_days: None,
                            description: desc,
                            goal: goal,
                            nodes: HashMap::new(),
                            metrics: fresh_metrics,
                            active: true,
                        }
                    },
                }
            },
            None => {
                return HabitData { 
                    start_year: today.year() as u32,
                    start_month: today.month(),
                    start_day: today.day(),
                    enabled_days: None,
                    description: desc,
                    goal: goal,
                    nodes: HashMap::new(),
                    metrics: fresh_metrics,
                    active: true,
                }
            }
        }
    }

    pub fn get_current_date_id() -> String {
        let current_date = chrono::Local::now();
        let date_id = format!("{}-{}-{}",
            current_date.month().to_string(),
            current_date.day().to_string(),
            current_date.year().to_string());
        return date_id;
    }

    pub fn validate_allowed_days(s: String) -> Result<Vec<u32>, String> {
        for x in s.split("-").map(|s| s.parse::<u32>()).collect::<Vec<Result<u32, ParseIntError>>>().iter() {
            match x {
                Ok(_) => {},
                Err(_) => {
                    return Err("Error: allowed_days argument was an invalid string.".to_string())
                },
            }
        };
        
        let parts = s.split("-").map(|s| s.parse::<u32>().unwrap()).collect::<Vec<u32>>();
        return Ok(parts);
    }

    pub fn insert_fresh_node(&mut self, date: String) -> Result<String, String> {
        let exists = self.nodes.get(&date);
        match exists {
            Some(_) => {
                Err("Node already exists for this habit on the specified date!".to_string())
            },
            None => {
                self.nodes.insert(date.clone(), self.create_node_from_habit());
                let _= self.shift_metric(None, Some(NodeStatus::IDLE));
                Ok(date)
            }
        }
    }

    pub fn create_node_from_habit(&self) -> HabitNode {
        HabitNode {
            value: 0,
            goal: self.goal,
            status: NodeStatus::IDLE,
        }
    }

    pub fn edit_node(&mut self, day: String, command: &str, value: i32) -> Result<String, String> {
        match self.nodes.get_mut(&day) {
            Some(node) => {
                let current_status = node.status.clone();
                match command {
                    "complete" => {
                        node.complete_node();
                        let _ = self.shift_metric(Some(current_status), Some(NodeStatus::COMPLETE));
                        Ok("".to_string())
                    },
                    "fail" => {
                        node.fail_node();
                        let _ = self.shift_metric(Some(current_status), Some(NodeStatus::FAILED));
                        Ok("".to_string())
                    },
                    "set" => {
                        node.value = value;
                        let new_status = node.calculate_status();
                        if new_status != current_status {
                            let _ = self.shift_metric(Some(current_status), Some(new_status));
                        }
                        Ok("".to_string())
                    },
                    "skip" => {
                        node.skip_node();
                        let _ = self.shift_metric(Some(current_status), Some(NodeStatus::SKIPPED));
                        Ok("".to_string())
                    },
                    "reset" => {
                        node.idle_node();
                        let _ = self.shift_metric(Some(current_status), Some(NodeStatus::IDLE));
                        Ok("".to_string())
                    },
                    "increment" => {
                        node.value += value;
                        let new_status = node.calculate_status();
                        if new_status != current_status {
                            let _ = self.shift_metric(Some(current_status), Some(new_status));
                        }
                        Ok("".to_string())
                    },
                    _ => {
                        Err("Incorrect input to edit_node()".to_string())
                    }
                }
            },
            None => {
                match self.insert_fresh_node(day.clone()) {
                    Ok(_) => {
                        let node = self.nodes.get_mut(&day).unwrap();
                        match command {
                            "complete" => {
                                node.complete_node();
                                let _ = self.shift_metric(Some(NodeStatus::IDLE), Some(NodeStatus::COMPLETE));
                                Ok("".to_string())
                            },
                            "fail" => {
                                node.fail_node();
                                let _ = self.shift_metric(Some(NodeStatus::IDLE), Some(NodeStatus::FAILED));
                                Ok("".to_string())
                            },
                            "set" => {
                                node.value = value;
                                let new_status = node.calculate_status();
                                if new_status != NodeStatus::IDLE {
                                    let _ = self.shift_metric(Some(NodeStatus::IDLE), Some(new_status));
                                }
                                Ok("".to_string())
                            },
                            "skip" => {
                                node.skip_node();
                                let _ = self.shift_metric(Some(NodeStatus::IDLE), Some(NodeStatus::SKIPPED));
                                Ok("".to_string())
                            },
                            "reset" => {
                                node.idle_node();
                                Ok("".to_string())
                            },
                            "increment" => {
                                node.value += value;
                                let new_status = node.calculate_status();
                                if new_status != NodeStatus::IDLE {
                                    let _ = self.shift_metric(Some(NodeStatus::IDLE), Some(new_status));
                                }
                                Ok("".to_string())
                            },
                            _ => {
                                Err("Incorrect input to edit_node()".to_string())
                            }
                        }
                    },
                    Err(e) => {
                        println!("{}", e);
                        Err(e)
                    },
                }
            },
        }
    }

    pub fn shift_metric(&mut self, decrement: Option<NodeStatus>, increment: Option<NodeStatus>) -> Result<String, String> {
        match increment {
            Some(inc_status) => {
                if let Some(dec_status) = decrement {
                    *self.metrics.get_mut(&dec_status).unwrap() -= 1;
                    *self.metrics.get_mut(&inc_status).unwrap() += 1;
                    Ok("".to_string())
                } else {
                    *self.metrics.get_mut(&inc_status).unwrap() += 1;
                    Ok("".to_string())
                }
            },
            None => {
                Err("Invalid metric increment argument.".to_string())
            },
        }
    }

    pub fn print_metrics(&self) {
        let partial = *self.metrics.get(&NodeStatus::PARTIAL).unwrap() as f64 / (2*self.nodes.len()) as f64;
        let complete = *self.metrics.get(&NodeStatus::COMPLETE).unwrap() as f64 / self.nodes.len() as f64;
        let overall = (complete + partial) * 100 as f64;
        let overall_count = *self.metrics.get(&NodeStatus::COMPLETE).unwrap() as f64 + (*self.metrics.get(&NodeStatus::PARTIAL).unwrap() as f64/2 as f64);
        println!("Overall habit score: {:.1}% ({:.1}/{:?})", overall, overall_count, self.nodes.len() as i32);
        println!("Number of completed days: {:?}", self.metrics.get(&NodeStatus::COMPLETE).unwrap());
        println!("Number of partially completed days: {:?}", self.metrics.get(&NodeStatus::PARTIAL).unwrap());
        println!("Number of skipped days: {:?}", self.metrics.get(&NodeStatus::SKIPPED).unwrap());
        println!("Number of idle days: {:?}", self.metrics.get(&NodeStatus::IDLE).unwrap());
        println!("Number of failed days: {:?}", self.metrics.get(&NodeStatus::FAILED).unwrap());
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserData {
    id: u32,
    name: String,
    data: HashMap<String, HabitData>,
}

impl UserData {
    pub fn new() -> Self {
        UserData { id: 0, name: "".to_string(), data: HashMap::new() }
    }

    pub fn clear_data(&mut self) {
        self.data = HashMap::new();
    }

    pub fn show_history(&self, habit: String) -> Result<String, String> {
        match self.data.get(&habit) {
            Some(data) => {
                data.print_metrics();
                Ok("".to_string())
            },
            None => {
                Err("Couldn't find specified habit!".to_string())
            },
        }
    }

    pub fn add_habit(&mut self, name: String, data: HabitData) -> Result<String, String> {
        match self.data.get(&name) {
            Some(_) => {
                Err("Habit already exists with that name!".to_string())
            },
            None => {
                self.data.insert(name, data);
                Ok("".to_string())
            },
        }
    }

    pub fn remove_habit(&mut self, name: String) -> Result<String, String> {
        match self.data.remove(&name) {
            Some(_) => {
                Ok("".to_string())
            },
            None => {
                Err("No habit exists with that name!".to_string())
            },
        }
    }

    pub fn hide_habit(&mut self, name: String) -> Result<String, String> {
        match self.data.get_mut(&name) {
            Some(habit) => {
                habit.active = !habit.active;
                Ok("".to_string())
            },
            None => {
                Err("No habit with that name exists!".to_string())
            },
        }
    }

    pub fn edit_habit_node(&mut self, args: Vec<String>, date: String, value: i32) -> Result<String, String> {
        if let Some(habit) = args.get(2).map(|s| s.to_string()) {
            match self.data.get_mut(&habit) {
                Some(data) => {
                    data.edit_node(date, &args[1], value)
                },
                None => {
                    Err("Error: Cannot find data for the specified habit.".to_string())
                },
            }
        } else {
            Err("Error: Invalid habit input.".to_string())
        }
    }

    pub fn habit_list_for_day(&mut self, date: String) -> Result<String, String> {
        if self.data.len() == 0 {
            return Err("No habits to list!".to_string())
        }

        let day;
        if date == "".to_string() {
            day = HabitData::get_current_date_id();
        } else {
            day = date;
        }

        println!("Habit list for {}", day);
        for (key, value) in self.data.iter() {
            if value.active == true {
                match value.nodes.get(&day) {
                    Some(node) => {
                        println!("{}: {:?} ({}/{})", key, node.status, node.value, node.goal);
                    },
                    None => {
                        // Do nothing?
                    }
                }
            }
        }
        Ok("".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_makes_habit_user() {
        let test_user = UserData {
            id: 7,
            name: "Ricardo".to_string(),
            data: HashMap::new(),
        };
        assert_eq!(test_user.id, 007);
    }

    #[test]
    fn it_adds_data_to_user() {
        let mut test_user = UserData {
            id: 7,
            name: "Ricardo".to_string(),
            data: HashMap::new(),
        };
        let test_data = HabitData {
            start_year: 2023,
            start_month: 1,
            start_day: 1,
            enabled_days: Some(vec![0, 1, 5, 6]),
            description: "this is a test habit".to_string(),
            goal: 100,
            nodes: HashMap::new(),
            metrics: HashMap::new(),
            active: true,
        };
        test_user.data.insert("test_habit".to_string(), test_data.clone());
        let data_check = test_user.data.get(&"test_habit".to_string()).unwrap();
        assert_eq!(data_check.enabled_days, Some(vec![0, 1, 5, 6]));
    }
    #[test]
    fn it_adds_and_edits_node() {
        let mut test_user = UserData {
            id: 7,
            name: "Ricardo".to_string(),
            data: HashMap::new(),
        };
        let mut test_data = HabitData {
            start_year: 2023,
            start_month: 1,
            start_day: 1,
            enabled_days: Some(vec![0, 1, 5, 6]),
            description: "this is a test habit".to_string(),
            goal: 100,
            nodes: HashMap::new(),
            metrics: HashMap::new(),
            active: true,
        };
        let test_node = HabitNode {
            value: 10,
            goal: 100,
            status: NodeStatus::SKIPPED
        };
        test_data.nodes.insert("10-4-2023".to_string(), test_node);
        test_user.data.insert("test_habit".to_string(), test_data.clone());
        assert_eq!(test_user.data.get(&"test_habit".to_string()).unwrap().nodes.get("10-4-2023").unwrap().status, NodeStatus::SKIPPED);

        let node_check = test_user.data.get_mut(&"test_habit".to_string()).unwrap().nodes.get_mut("10-4-2023").unwrap();
        node_check.calculate_status();

        assert_eq!(test_user.data.get(&"test_habit".to_string()).unwrap().nodes.get("10-4-2023").unwrap().status, NodeStatus::PARTIAL);
    }

}