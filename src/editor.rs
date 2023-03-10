mod compiler;
mod default_os;
use compiler::*;
use std::{
    env::set_current_dir,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

const DEFAULT_PROGRAM: &str =
    "; To run a program, press the Compile button and then change to Run view.

; This program calculates 2+2

LOAD R1, =2     ; Load 2 into Register 1
ADD  R1, =2     ; Add 2 to R1
OUT  R1, =CRT   ; Output the value of R1

; Remember to stop the machine once you're done!
SVC SP, =HALT   ; Service call for halt.";

pub struct Editor {
    pub source_path: Option<String>,
    pub source_code: String,
    pub line_no: String,
    pub linecnt: i32,
    pub compiler: Compiler,
}

impl Default for Editor {
    fn default() -> Self {
        let mut ed = Editor {
            source_path: None,
            source_code: DEFAULT_PROGRAM.into(),
            line_no: "".into(),
            linecnt: 1,
            compiler: Compiler::default(),
        };
        ed.update_linecount();
        ed
    }
}

impl Editor {
    pub fn update_linecount(&mut self) {
        self.linecnt = self.source_code.matches("\n").count() as i32;
        self.line_no = "1".into();
        for i in 2..self.linecnt + 2 {
            self.line_no += "\n";
            self.line_no += i.to_string().as_str();
        }
    }

    pub fn compile(&mut self) -> Result<String, ()> {
        self.compiler.compile(self.source_code.clone())
    }

    pub fn compile_default_os(&mut self) -> Result<String, ()> {
        self.compiler
            .compile(format!("org {};\n", default_os::DEFAULT_SVC_ORG) + default_os::DEFAULT_OS)
    }

    pub fn open_file(&mut self, pathbuf: Option<PathBuf>) {
        match pathbuf {
            None => return,
            Some(filepath) => {
                let file = fs::read_to_string(filepath.clone());
                if let Ok(s) = file {
                    self.source_code = s;
                    set_current_dir(PathBuf::from(&filepath).parent().unwrap());
                    self.source_path = Some(filepath.to_str().unwrap().into());
                    self.update_linecount();
                }
            }
        }
    }

    pub fn save_file(&mut self, pathbuf: Option<PathBuf>) {
        let filepath: String;
        match pathbuf {
            Some(s) => filepath = s.to_str().unwrap().into(),
            None => match self.source_path.clone() {
                Some(s) => filepath = s,
                None => panic!("editor.rs, save_file(): attempted to save, no filepath"),
            },
        }
        let file = File::create(&filepath);
        if let Ok(mut f) = file {
            f.write_all(self.source_code.as_bytes());
            set_current_dir(PathBuf::from(&filepath).parent().unwrap());
            self.source_path = Some(filepath);
        }
    }
}
