use std::{collections::BTreeMap, fs};

#[derive(Debug)]
pub struct TaskRow {
    pub kind: String,        // "send_email"
    pub priority: u8,        // 5
    pub max_attempts: u8,    // 3
}

pub fn parse_line(line: &str) -> Option<TaskRow> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 3 {
        return None;
    }
    Some(TaskRow {
        kind: fields[0].trim().to_string(),
        priority: fields[1].trim().parse().ok()?,
        max_attempts: fields[2].trim().parse().ok()?,
    })
}

pub fn parse_tasks(text: &str) -> (Vec<TaskRow>, Vec<usize>) {
    let mut tasks = Vec::new();
    let mut bad_lines = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match parse_line(line) {
            Some(task) => tasks.push(task),
            None => bad_lines.push(index + 1),
        }
    }
    (tasks, bad_lines)
}

pub fn count_by_kind(tasks: &[TaskRow]) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::new();
    for task in tasks {
        *counts.entry(task.kind.clone()).or_insert(0) += 1;
    }
    counts
}


fn main() {
    let path = std::env::args().nth(1).expect("No file were provided");
    let raw_tasks = fs::read_to_string(path).expect("Failed to read a file");
    let (tasks, _) = parse_tasks(&raw_tasks);
    let counts = count_by_kind(&tasks);
    let mut kinds: Vec<String> = counts.keys().cloned().collect();
    kinds.sort();
    println!("{:?}", kinds);
    for kind in kinds {
        println!("{kind:<16} {}", counts[&kind]);
    }
}

#[test]
fn rejects_bad_lines() {
    assert!(parse_line("generate_report,high,2").is_none());
    assert!(parse_line("resize_image,2").is_none());
}