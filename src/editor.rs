mod compiler;
use compiler::*;
use std::{
    env::set_current_dir,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

const DEFAULT_OS: &str = include_str!("../samples/default_os.k91");
const DEFAULT_PROGRAM: &str = include_str!("../samples/default_program.k91");

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
        self.compiler.compile(DEFAULT_OS.into())
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
