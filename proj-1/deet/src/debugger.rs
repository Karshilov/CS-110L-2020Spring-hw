use crate::debugger_command::DebuggerCommand;
use crate::inferior::{Inferior, Status};
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
        }
    }

    pub fn create_inferior(&mut self, args: Vec<String>) {
        if let Some(inferior) = Inferior::new(&self.target, &args) {
            self.inferior = Some(inferior);
            let inferior = self.inferior.as_mut().unwrap();
            let res = inferior.cont_process();
            match res {
                Ok(value) => match value {
                    Status::Stopped(sign, pts) => {
                        println!("The stop-signal is {} and it has executed at {}", sign, pts);
                    }
                    Status::Exited(code) => {
                        println!("The program exited with code {}", code);
                    }
                    Status::Signaled(sign) => {
                        println!("The program stop by sign {}", sign);
                    }
                },
                Err(err) => {
                    println!("{}", err);
                }
            }
        } else {
            println!("Error starting subprocess");
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => match &mut self.inferior {
                    Some(value) => {
                        println!("Killing running inferior (pid {})", value.pid());
                        match value.kill() {
                            Ok(_) => {}
                            Err(err) => {
                                println!("Fail to kill the running inferior: {}", err);
                            }
                        }
                        self.create_inferior(args);
                    }
                    None => {
                        self.create_inferior(args);
                    }
                },
                DebuggerCommand::Quit => match &mut self.inferior {
                    Some(value) => {
                        println!("Killing running inferior (pid {})", value.pid());
                        match value.kill() {
                            Ok(_) => {
                                return;
                            }
                            Err(err) => {
                                println!("Fail to kill the running inferior: {}", err);
                            }
                        }
                    }
                    None => {
                        return;
                    }
                },
                DebuggerCommand::Continue => match self.inferior {
                    Some(_) => {
                        let inferior = self.inferior.as_mut().unwrap();
                        let res = inferior.cont_process();
                        match res {
                            Ok(value) => match value {
                                Status::Stopped(sign, pts) => {
                                    println!(
                                        "The stop-signal is {} and it has executed at {}",
                                        sign, pts
                                    );
                                }
                                Status::Exited(code) => {
                                    println!("The program exited with code {}", code);
                                }
                                Status::Signaled(sign) => {
                                    println!("The program stop by sign {}", sign);
                                }
                            },
                            Err(err) => {
                                println!("{}", err);
                            }
                        }
                    }
                    None => {
                        println!("ERROR! There's no program is running.");
                    }
                },
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
