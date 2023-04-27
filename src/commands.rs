use alloc::string::String;

use crate::{print, println, task::keyboard::AppContext};

pub enum Commands {
    Help,
    Echo { message: String },
    Prompt { prompt: String },
    Unknown { command: String },
    Draw,
}

impl Commands {
    pub fn from_str(context: &mut AppContext) -> Self {
        // Separate the command from the arguments
        let command = context
            .command_cache
            .split_whitespace()
            .next()
            .unwrap_or("");

        let args = context
            .command_cache
            .strip_prefix(command)
            .unwrap_or("")
            .strip_prefix(" ")
            .unwrap_or("");

        match command {
            "help" => Commands::Help,
            "echo" => Commands::Echo {
                message: String::from(args),
            },
            "prompt" => Commands::Prompt {
                prompt: String::from(args),
            },
            "draw" => Commands::Draw,
            _ => Commands::Unknown {
                command: context.command_cache.clone(),
            },
        }
    }

    pub fn execute(&self, context: &mut AppContext) {
        print!("\n");
        match self {
            Commands::Help => {
                println!("Available commands:");
                println!("help - display this help message");
                println!("echo - echo the command cache");
            }
            Commands::Echo { message } => {
                println!("{}", message)
            }
            Commands::Prompt { prompt } => {
                context.prompt = String::from(prompt);
            }
            Commands::Draw => {}
            Commands::Unknown { command } => {
                println!("Unknown command: {}", command);
            }
        }
    }
}

pub fn handle_command(context: &mut AppContext) {
    let command = Commands::from_str(context);
    command.execute(context);
    print!("{}", context.prompt);
    context.command_cache.clear();
}
