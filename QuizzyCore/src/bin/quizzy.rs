use clap::{Parser, Subcommand};
use quizzy_core::*;

#[derive(Parser)]
#[command(name = "quizzy", about = "QuizzyCore teacher CLI", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new problem set interactively and output JSON
    Create {
        /// Output file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Encode a problem set JSON into a distributable format
    EncodeProblems {
        /// Input problem set JSON file
        input: String,
        /// Quiz key for encrypting the answer key
        #[arg(short, long)]
        quiz_key: String,
        /// Output file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Encrypt a problem set JSON for student distribution (hides answers)
    Package {
        /// Input problem set JSON file (with quizKey field)
        input: String,
        /// Output file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Decrypt an encrypted problem set back to JSON
    Unpackage {
        /// Input encrypted problem set file
        input: String,
        /// Quiz key used to encrypt
        #[arg(short, long)]
        quiz_key: String,
        /// Output JSON file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Decode just the student session (no quiz key needed)
    DecodeSession {
        /// Input encoded blob file (or paste the blob directly)
        input: String,
        /// Output JSON file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Decode the answer key (requires quiz key)
    DecodeAnswers {
        /// Input encoded blob file
        input: String,
        /// Quiz key used when the quiz was created
        #[arg(short, long)]
        quiz_key: String,
        /// Output JSON file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Decode both session and answer key (requires quiz key)
    DecodeAll {
        /// Input encoded blob file
        input: String,
        /// Quiz key used when the quiz was created
        #[arg(short = 'k', long)]
        quiz_key: String,
        /// Output JSON file (writes to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Decode and grade: print a grading summary (requires quiz key)
    Grade {
        /// Input encoded blob file
        input: String,
        /// Quiz key used when the quiz was created
        #[arg(short, long)]
        quiz_key: String,
    },
}

fn read_input(input: &str) -> String {
    // Check if input is a file path
    if std::path::Path::new(input).exists() {
        std::fs::read_to_string(input).unwrap_or_else(|_| input.to_string())
    } else {
        input.to_string()
    }
}

fn write_output(output: &Option<String>, content: &str) {
    if let Some(path) = output {
        std::fs::write(path, content).expect("Failed to write output file");
        println!("Wrote to {}", path);
    } else {
        println!("{}", content);
    }
}

// ---------- Interactive problem set creation ----------

fn interactive_create() -> types::ProblemSet {
    use std::io::{self, Write};

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("=== Quizzy Problem Set Creator ===\n");

    print!("Student ID: ");
    stdout.flush().unwrap();
    let mut student_id = String::new();
    stdin.read_line(&mut student_id).unwrap();
    let student_id = student_id.trim().to_string();

    print!("Quiz Key (random string to encrypt answer key): ");
    stdout.flush().unwrap();
    let mut quiz_key = String::new();
    stdin.read_line(&mut quiz_key).unwrap();
    let quiz_key = quiz_key.trim().to_string();

    let mut problems = Vec::new();
    let mut id_counter: u64 = 1;

    loop {
        println!("\n--- Problem #{} ---", id_counter);

        print!("Points: ");
        stdout.flush().unwrap();
        let mut points_str = String::new();
        stdin.read_line(&mut points_str).unwrap();
        let points: u64 = points_str.trim().parse().unwrap_or(10);

        println!("Prompt (Markdown, end with a blank line):");
        let mut prompt = String::new();
        loop {
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();
            if line.trim().is_empty() && !prompt.is_empty() {
                break;
            }
            prompt.push_str(&line);
        }
        let prompt = prompt.trim().to_string();

        println!("Variant type:");
        println!("  1. MCQ  — multiple choice (single answer)");
        println!("  2. MMCQ — multiple multiple-choice (select all)");
        println!("  3. FRQ  — free response (auto-graded)");
        println!("  4. FFRQ — free-form response (human-graded)");
        print!("Choice [1-4]: ");
        stdout.flush().unwrap();
        let mut choice = String::new();
        stdin.read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                let mut options = Vec::new();
                println!("Enter options (one per line, blank line to finish):");
                for label in &['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'] {
                    print!("  {}. ", label);
                    stdout.flush().unwrap();
                    let mut opt = String::new();
                    stdin.read_line(&mut opt).unwrap();
                    if opt.trim().is_empty() {
                        break;
                    }
                    options.push(opt.trim().to_string());
                }
                print!("Correct option index (0-based): ");
                stdout.flush().unwrap();
                let mut idx_str = String::new();
                stdin.read_line(&mut idx_str).unwrap();
                let correct_idx: usize = idx_str.trim().parse().unwrap_or(0);

                problems.push(types::Problem {
                    id: id_counter,
                    points,
                    prompt,
                    variant: types::ProblemVariant::MCQ { options, correct_idx },
                });
            }
            "2" => {
                let mut options = Vec::new();
                println!("Enter options (one per line, blank line to finish):");
                for label in &['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'] {
                    print!("  {}. ", label);
                    stdout.flush().unwrap();
                    let mut opt = String::new();
                    stdin.read_line(&mut opt).unwrap();
                    if opt.trim().is_empty() {
                        break;
                    }
                    options.push(opt.trim().to_string());
                }
                print!("Correct option indices (0-based, space-separated): ");
                stdout.flush().unwrap();
                let mut idx_str = String::new();
                stdin.read_line(&mut idx_str).unwrap();
                let correct_idxs: Vec<usize> = idx_str
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();

                problems.push(types::Problem {
                    id: id_counter,
                    points,
                    prompt,
                    variant: types::ProblemVariant::MMCQ { options, correct_idxs },
                });
            }
            "3" => {
                print!("Expected pattern: ");
                stdout.flush().unwrap();
                let mut pattern = String::new();
                stdin.read_line(&mut pattern).unwrap();

                println!("Match strategy:");
                println!("  1. exact");
                println!("  2. contains");
                println!("  3. regex");
                print!("Choice [1-3]: ");
                stdout.flush().unwrap();
                let mut strat_choice = String::new();
                stdin.read_line(&mut strat_choice).unwrap();
                let strategy = match strat_choice.trim() {
                    "1" => types::MatchStrategy::Exact,
                    "2" => types::MatchStrategy::Contains,
                    _ => types::MatchStrategy::Regex,
                };

                problems.push(types::Problem {
                    id: id_counter,
                    points,
                    prompt,
                    variant: types::ProblemVariant::FRQ {
                        expected_pattern: pattern.trim().to_string(),
                        strategy,
                    },
                });
            }
            "4" => {
                problems.push(types::Problem {
                    id: id_counter,
                    points,
                    prompt,
                    variant: types::ProblemVariant::FFRQ,
                });
            }
            _ => {
                println!("Invalid choice, skipping.");
                continue;
            }
        }

        id_counter += 1;

        print!("\nAdd another problem? [y/N]: ");
        stdout.flush().unwrap();
        let mut more = String::new();
        stdin.read_line(&mut more).unwrap();
        if !more.trim().to_lowercase().starts_with('y') {
            break;
        }
    }

    types::ProblemSet {
        student_id,
        quiz_key,
        problems,
    }
}

// ---------- Main ----------

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Create { output } => {
            let ps = interactive_create();
            let json = to_json(&ps).expect("serialization must succeed");
            write_output(&output, &json);
            println!("\nProblem set created with {} problems.", ps.problems.len());
        }
        Command::EncodeProblems {
            input,
            quiz_key: _,
            output,
        } => {
            let raw = read_input(&input);
            let ps: types::ProblemSet =
                from_json(&raw).expect("Failed to parse problem set JSON");
            let json = to_json(&ps).expect("serialization must succeed");
            write_output(&output, &json);
            eprintln!("Encoded problem set ({} problems).", ps.problems.len());
        }
        Command::Package {
            input,
            output,
        } => {
            let raw = read_input(&input);
            let ps: types::ProblemSet = from_json(&raw).expect("Failed to parse problem set JSON");
            let sps = package_student_safe(&ps);
            let json = serde_json::to_string_pretty(&sps).expect("serialization must succeed");
            write_output(&output, &json);
            eprintln!(
                "Student-safe problem set ({} problems). Answers stripped. Share this JSON with students.",
                sps.problems.len()
            );
        }
        Command::Unpackage {
            input,
            quiz_key,
            output,
        } => {
            let raw = read_input(&input).trim().to_string();
            match decrypt_problem_set(&raw, &quiz_key) {
                Ok(json) => {
                    write_output(&output, &json);
                    eprintln!("Decrypted successfully.");
                }
                Err(e) => {
                    eprintln!("Decrypt error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::DecodeSession { input, output } => {
            let raw = read_input(&input).trim().to_string();
            match decode_session(&raw) {
                Ok(session) => {
                    let json =
                        serde_json::to_string_pretty(&session).expect("serialization must succeed");
                    write_output(&output, &json);
                }
                Err(e) => {
                    eprintln!("Decode error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::DecodeAnswers {
            input,
            quiz_key,
            output,
        } => {
            let raw = read_input(&input).trim().to_string();
            match decode_answer_key(&raw, &quiz_key) {
                Ok(answer_key) => {
                    let json = serde_json::to_string_pretty(&answer_key)
                        .expect("serialization must succeed");
                    write_output(&output, &json);
                }
                Err(e) => {
                    eprintln!("Decode error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::DecodeAll {
            input,
            quiz_key,
            output,
        } => {
            let raw = read_input(&input).trim().to_string();
            match decode_all(&raw, &quiz_key) {
                Ok(decoded) => {
                    let json = serde_json::to_string_pretty(&decoded)
                        .expect("serialization must succeed");
                    write_output(&output, &json);
                }
                Err(e) => {
                    eprintln!("Decode error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Grade { input, quiz_key } => {
            let raw = read_input(&input).trim().to_string();
            match decode_all(&raw, &quiz_key) {
                Ok(decoded) => {
                    println!("=== Grading Report ===");
                    println!("Student: {}", decoded.session.student_id);
                    println!(
                        "Completed: {}",
                        chrono_like_fmt(&decoded.session.timestamp)
                    );
                    println!();

                    let total_points: u64 = decoded.answer_key.values().map(|e| match e {
                        types::AnswerKeyEntry::MCQ { points, .. } => points,
                        types::AnswerKeyEntry::MMCQ { points, .. } => points,
                        types::AnswerKeyEntry::FRQ { points, .. } => points,
                        types::AnswerKeyEntry::FFRQ { points, .. } => points,
                    }).sum();

                    let mut earned = 0u64;
                    let mut auto_possible = 0u64;
                    let mut pending = 0u64;

                    for entry in &decoded.session.entries {
                        let key_entry = decoded.answer_key.get(&entry.problem_id);
                        let max_points = key_entry.map(|e| match e {
                            types::AnswerKeyEntry::MCQ { points, .. } => points,
                            types::AnswerKeyEntry::MMCQ { points, .. } => points,
                            types::AnswerKeyEntry::FRQ { points, .. } => points,
                            types::AnswerKeyEntry::FFRQ { points, .. } => points,
                        }).unwrap_or(&0);

                        let status = match entry.grading_status {
                            types::GradingStatus::AutoGraded => {
                                auto_possible += max_points;
                                earned += entry.points_awarded;
                                if entry.is_correct == Some(true) {
                                    "✓ CORRECT"
                                } else {
                                    "✗ INCORRECT"
                                }
                            }
                            types::GradingStatus::PendingHuman => {
                                pending += 1;
                                "👤 PENDING HUMAN"
                            }
                        };

                        let timed = format!(
                            "{:.1}s",
                            entry.time_spent_ms as f64 / 1000.0
                        );

                        println!(
                            "  #{}  {:14}  {:>4}/{:>4} pts  {}",
                            entry.problem_id, status, entry.points_awarded, max_points, timed
                        );
                    }

                    println!();
                    println!("─────────────────────────────────────");
                    println!(
                        "  Auto-graded: {}/{} pts",
                        earned, auto_possible
                    );
                    if pending > 0 {
                        println!(
                            "  Pending human grading: {} question{} ({} pts outstanding)",
                            pending,
                            if pending > 1 { "s" } else { "" },
                            total_points - auto_possible - earned
                        );
                    }
                    println!(
                        "  Total: {}/{} pts",
                        earned,
                        total_points
                    );
                }
                Err(e) => {
                    eprintln!("Decode error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn chrono_like_fmt(ts: &u64) -> String {
    let _secs = *ts / 1000;
    format!("timestamp={} ms", ts)
}
