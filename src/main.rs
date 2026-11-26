use std::{
    collections::{BTreeMap, HashSet},
    fs,
};

#[derive(Debug)]
pub struct TaskRow {
    pub kind: String,     // "send_email"
    pub priority: u8,     // 5
    pub max_attempts: u8, // 3
}

pub fn parse_line(line: &str) -> Result<TaskRow, String> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 3 {
        return Err(format!("expected 3 fields, got {}", fields.len()));
    }
    let kind = fields[0].trim();
    if kind.is_empty() {
        return Err("kind is empty".to_string());
    }
    let priority: u8 = match fields[1].trim().parse() {
        Ok(p) if p > 0 => p,
        _ => return Err(format!("bad priority '{}'", fields[1].trim())),
    };
    let max_attempts: u8 = match fields[2].trim().parse() {
        Ok(m) if m > 0 => m,
        _ => return Err(format!("bad max attempts '{}'", fields[2].trim())),
    };
    Ok(TaskRow {
        kind: kind.to_string(),
        priority,
        max_attempts,
    })
}

pub fn parse_tasks(text: &str) -> (Vec<TaskRow>, Vec<(usize, String)>) {
    let mut tasks = Vec::new();
    let mut bad_lines = Vec::new();
    let mut seen = HashSet::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        match parse_line(line) {
            Ok(task) => {
                // в файле нет id, поэтому дубликат = такая же строка целиком
                let key = (task.kind.clone(), task.priority, task.max_attempts);
                if seen.insert(key) {
                    tasks.push(task);
                } else {
                    bad_lines.push((index + 1, "duplicate".to_string()));
                }
            }
            Err(reason) => bad_lines.push((index + 1, reason)),
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
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 || args[1] != "preview" {
        eprintln!("Usage: job-queue preview <file>");
        std::process::exit(1);
    }
    let raw_tasks = fs::read_to_string(&args[2]).expect("Failed to read a file");
    let (tasks, bad_lines) = parse_tasks(&raw_tasks);

    for (line, reason) in &bad_lines {
        println!("line {line}: {reason}");
    }

    println!("total:  {}", tasks.len() + bad_lines.len());
    println!("valid:  {}", tasks.len());
    println!("errors: {}", bad_lines.len());
    for (kind, count) in count_by_kind(&tasks) {
        println!("{kind:<20} {count}");
    }
}

#[test]
fn parses_good_line() {
    let task = parse_line(" email_send , 5 , 3").unwrap();
    assert_eq!(task.kind, "email_send");
    assert_eq!(task.priority, 5);
    assert_eq!(task.max_attempts, 3);
}

#[test]
fn rejects_bad_lines() {
    assert!(parse_line("generate_report,high,2").is_err());
    assert!(parse_line("resize_image,2").is_err());
    assert!(parse_line("resize_image,2,3,4").is_err());
    assert!(parse_line(",5,3").is_err());
    assert!(parse_line("cleanup,0,1").is_err());
    assert!(parse_line("cleanup,-1,1").is_err());
    assert!(parse_line("cleanup,256,1").is_err());
    assert!(parse_line("webhook,7,many").is_err());
}

#[test]
fn skips_duplicates_and_keeps_line_numbers() {
    let text = "a,1,1\n\na,1,1\nb,2,2\nbroken\n";
    let (tasks, bad_lines) = parse_tasks(text);
    assert_eq!(tasks.len(), 2);
    assert_eq!(bad_lines.len(), 2);
    assert_eq!(bad_lines[0], (3, "duplicate".to_string()));
    assert_eq!(bad_lines[1].0, 5);
}

#[test]
fn counts_by_kind() {
    let (tasks, _) = parse_tasks("a,1,1\nb,1,1\na,2,1\n");
    let counts = count_by_kind(&tasks);
    assert_eq!(counts["a"], 2);
    assert_eq!(counts["b"], 1);
}
