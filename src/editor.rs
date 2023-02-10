mod compiler;
use std::{
    env::current_dir,
    fs::{self, File},
    io::Write,
    path::{self, PathBuf},
};

use compiler::*;

const default_program: &str =
    "; To run a program, press the Compile button and then change to Run view.

; This program calculates 2+2

LOAD R1, =2     ; Load 2 into Register 1
ADD  R1, =2     ; Add 2 to R1
OUT  R1, =CRT   ; Output the value of R1

; Remember to stop the machine once you're done!
SVC SP, =HALT   ; Service call for halt.";

pub struct Editor {
    pub working_dir: String,
    pub source_path: Option<String>,
    pub source_code: String,
    pub line_no: String,
}

impl Default for Editor {
    fn default() -> Self {
        let mut ed = Editor {
            working_dir: current_dir().unwrap().display().to_string(),
            source_path: None,
            source_code: default_program.into(),
            line_no: "".into(),
        };
        ed.update_linecount();
        ed
    }
}

impl Editor {
    pub fn update_linecount(&mut self) {
        let mut linecnt = self.source_code.matches("\n").count();
        self.line_no = String::new();
        self.line_no = "1".into();

        for i in 2..linecnt + 2 {
            self.line_no += "\n";
            self.line_no += i.to_string().as_str();
        }
    }

    pub fn compile(&mut self) -> String {
        compile(self.source_code.clone())
    }

    pub fn open_file(&mut self, pathbuf: Option<PathBuf>) {
        match pathbuf {
            None => return,
            Some(filepath) => {
                let file = fs::read_to_string(filepath.clone());
                match file {
                    Err(_) => return,
                    Ok(s) => {
                        self.source_code = s;
                        std::env::set_current_dir(PathBuf::from(&filepath).parent().unwrap());
                        self.source_path = Some(filepath.to_str().unwrap().into());
                    }
                }
            }
        }
    }

    pub fn save_file(&mut self, pathbuf: Option<PathBuf>) {
        let filepath: String;
        match pathbuf {
            // Provided path
            Some(s) => filepath = s.to_str().unwrap().into(),
            None => {
                match self.source_path.clone() {
                    // No path provided, use currently loaded
                    Some(s) => filepath = s,
                    None => panic!("editor.rs, save_file(): attempted to save, no filepath"),
                }
            }
        }
        let file = File::create(&filepath);
        match file {
            Err(_) => return,
            Ok(mut f) => {
                f.write_all(self.source_code.as_bytes());
            }
        }
        std::env::set_current_dir(PathBuf::from(&filepath).parent().unwrap());
        self.source_path = Some(filepath);
    }
}
