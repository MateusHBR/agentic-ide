use portable_pty::{native_pty_system, CommandBuilder, PtySize, MasterPty};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

struct TerminalInstance {
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
}

pub struct TerminalManager {
    terminals: HashMap<String, TerminalInstance>,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            terminals: HashMap::new(),
        }
    }

    pub fn create(
        &mut self,
        cwd: &str,
        cmd: Option<&str>,
        app: AppHandle,
    ) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        let mut command = if let Some(cmd_str) = cmd {
            let mut c = CommandBuilder::new(&shell);
            c.arg("-l");
            c.arg("-c");
            c.arg(cmd_str);
            c
        } else {
            let mut c = CommandBuilder::new(&shell);
            c.arg("-l");
            c
        };
        command.cwd(cwd);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");

        let _child = pair.slave.spawn_command(command).map_err(|e| e.to_string())?;
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
        let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

        let term_id = id.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => {
                        let _ = app.emit(
                            "terminal-exit",
                            serde_json::json!({ "id": term_id }),
                        );
                        break;
                    }
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = app.emit(
                            "terminal-output",
                            serde_json::json!({
                                "id": term_id,
                                "data": data
                            }),
                        );
                    }
                    Err(_) => break,
                }
            }
        });

        self.terminals.insert(
            id.clone(),
            TerminalInstance {
                writer,
                master: pair.master,
            },
        );

        Ok(id)
    }

    pub fn write(&mut self, id: &str, data: &[u8]) -> Result<(), String> {
        let terminal = self.terminals.get_mut(id).ok_or("Terminal not found")?;
        terminal
            .writer
            .write_all(data)
            .map_err(|e| e.to_string())?;
        terminal.writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn resize(&self, id: &str, rows: u16, cols: u16) -> Result<(), String> {
        let terminal = self.terminals.get(id).ok_or("Terminal not found")?;
        terminal
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn close(&mut self, id: &str) -> Result<(), String> {
        self.terminals
            .remove(id)
            .ok_or("Terminal not found".to_string())?;
        Ok(())
    }
}

pub type TerminalState = Mutex<TerminalManager>;
