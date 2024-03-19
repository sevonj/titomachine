use std::env::set_current_dir;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use libttktk::b91::{B91, B91ParseError};
use libttktk::compiler::compile;

const DEFAULT_OS: &str = include_str!("../programs/default/default_os.k91");
const DEFAULT_PROGRAM: &str = include_str!("../programs/default/default_program.k91");

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct Editor {
    #[serde(skip)] pub(crate) source_path: Option<String>,
    #[serde(skip)] pub(crate) source_code: String,
    #[serde(skip)] pub(crate) compiler_output: String,
    #[serde(skip)] pub(crate) line_no: String,
    #[serde(skip)] pub(crate) linecnt: i32,
    #[serde(skip)] pub(crate) default_os: Option<B91>,
    pub(crate) compile_default_os: bool,
}

impl Default for Editor {
    fn default() -> Self {
        let mut editor = Editor {
            source_path: None,
            source_code: DEFAULT_PROGRAM.into(),
            compiler_output: String::new(),
            line_no: "".into(),
            linecnt: 1,
            compile_default_os: true,
            default_os: Some(compile(DEFAULT_OS.into()).unwrap().parse().unwrap()),
        };
        editor.update_linecount();
        editor
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

    pub fn compile(&mut self) -> Result<B91, String> {
        let result = compile(self.source_code.clone());
        if let Err(e) = &result {
            self.compiler_output = e.clone()
        }
        match B91::from_str(result.unwrap().as_str()) {
            Ok(b91) => Ok(b91),
            Err(e) => Err(format!("Compiler succeeded, but parser failed! Please file an issue. {e}"))
        }
    }

    /*
    pub fn compile_default_os(&mut self) -> Result<String, String> {
        let result = compile(DEFAULT_OS.into());
        if let Err(e) = &result {
            self.compiler_output = e.clone()
        }
        result
    }*/

    pub fn open_file(&mut self, pathbuf: Option<PathBuf>) {
        match pathbuf {
            None => return,
            Some(filepath) => {
                let file = fs::read_to_string(filepath.clone());
                if let Ok(s) = file {
                    self.source_code = s;
                    let _ = set_current_dir(PathBuf::from(&filepath).parent().unwrap());
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
            let _ = f.write_all(self.source_code.as_bytes());
            let _ = set_current_dir(PathBuf::from(&filepath).parent().unwrap());
            self.source_path = Some(filepath);
        }
    }
}

// Tests

#[cfg(test)]
mod test {
    use libttktk::compiler::compile;
    use libttktk::disassembler::disassemble_instruction;

    /// Compile different values in all bases
    #[test]
    fn test_compiler_variables() {
        let vec = editor_compile(include_str!("../programs/tests/test_compiler_variables.k91").into());
        for i in 0..=3 {
            let expected = 52;
            assert_eq!(vec[i], expected)
        }
        for i in 4..=11 {
            let expected = -1;
            assert_eq!(vec[i], expected)
        }
    }

    #[test]
    fn test_compiler_disassembler() {
        compile_disass_compile(include_str!("../programs/tests/test_compiler_opcodes.k91").into());
        compile_disass_compile(include_str!("../programs/tests/test_compiler_addressing.k91").into());
    }

    /// This function tests the compiler and the disassmbler against each other.
    /// Steps:
    /// 1. Compile test program
    /// 2. Disassemble the resulting binary
    /// 3. Compile disassembled code
    /// 4. Assert that both binaries are the same
    fn compile_disass_compile(source: String) {
        println!("Source code:");
        print_source(source.clone());

        let vec1 = editor_compile(source);
        let mut disassembled = String::new();

        for i in 0..vec1.len() {
            disassembled += disassemble_instruction(vec1[i]).as_str();
            disassembled += "\n";
        }

        println!("Disassembled code:");
        print_source(disassembled.clone());

        let vec2 = editor_compile(disassembled);
        println!("Comparing binaries compiled from source and disassembly:\nSource    Disassembly");
        for i in 0..vec1.len() {
            print!("{:08x}, {:08x}", vec1[i], vec2[i]);
            if vec1[i] != vec2[i] {
                println!(" <- Mismatch!");
                print_instruction(vec1[i]);
                print_instruction(vec2[i]);
            }
            println!();
        }
        assert_eq!(vec1, vec2);
    }

    /// Compile and return result as a Vec<i32>
    fn editor_compile(source: String) -> Vec<i32> {
        let compiled = match compile(source) {
            Ok(res) => res,
            Err(e) => panic!("{}", e)
        };
        let mut vec: Vec<i32> = Vec::new();
        let mut lines = compiled.lines();
        loop {
            match lines.next() {
                None => break,
                Some("___b91___") => {}
                Some("___code___") => {
                    lines.next();
                }
                Some("___data___") => {
                    lines.next();
                }
                Some("___symboltable___") => break,
                Some(val) => vec.push(val.parse::<i32>().unwrap()),
            }
        }
        vec
    }

    fn print_source(source: String) {
        let mut i = 1;
        source.lines().for_each(|line| {
            println!("line {}: {}", i, line);
            i += 1;
        });
    }

    fn print_instruction(ins: i32) {
        let opcode = (ins >> 24) as u16;
        let rj = (ins >> 21) & 0x7;
        let mode = (ins >> 19) & 0x3;
        let ri = (ins >> 16) & 0x7;
        let addr = (ins & 0xffff) as i16 as i32;
        println!(
            "{:04x}-{:03b}-{:03b}-{:03b}-{:08x}  {}",
            opcode,
            rj,
            mode,
            ri,
            addr,
            disassemble_instruction(ins)
        );
    }
}
