use super::super::GuiMode;
use crate::{emulator::emu_debug::CtrlMSG, TitoApp};
use rfd::FileDialog;
use std::{env::current_dir, path::PathBuf};

impl TitoApp {
    pub fn file_new(&mut self) {
        self.filestatus = FileStatus::default();
        self.editor.source_code = "".into();
        self.editor.source_path = None;
        self.editor.update_linecount();
        self.guimode = GuiMode::Editor;
    }

    pub fn file_open(&mut self) {
        let path = FileDialog::new()
            .add_filter("TTK Source files", &["k91"])
            .set_directory(&self.workdir)
            .pick_file();
        if path == None {
            return;
        }
        self.filestatus = FileStatus::new_with_path(path.clone());
        self.filestatus.update_title();
        self.editor.open_file(path);
        self.guimode = GuiMode::Editor;
        self.workdir = current_dir().unwrap();
    }

    pub fn file_save(&mut self) {
        println!("save called");
        // If new file, use save as because there's no filename yet.
        if self.filestatus.currentfile == None {
            self.file_saveas();
            return;
        }
        self.editor.save_file(None);
        self.filestatus.on_save();
    }

    pub fn file_saveas(&mut self) {
        let path = FileDialog::new()
            .add_filter("TTK Source files", &["k91"])
            .set_directory(&self.workdir)
            .save_file();
        if path == None {
            return;
        }
        self.editor.save_file(path);
        self.filestatus.on_save();
        self.workdir = current_dir().unwrap();
    }

    pub fn file_compile(&mut self) {
        self.tx_ctrl.send(CtrlMSG::ClearMem);

        // Compile Default OS
        if self.editorsettings.compile_default_os {
            if let Ok(prog) = self.editor.compile_default_os() {
                self.tx_ctrl.send(CtrlMSG::LoadProg(prog));
            } else {
                panic!("Failed to compile default OS!")
            }
        }
        // Compile the actual program
        if let Ok(prog) = self.editor.compile() {
            self.tx_ctrl.send(CtrlMSG::LoadProg(prog));
            self.filestatus.on_compile(Ok(()));
            self.guimode = GuiMode::Emulator;
        } else {
            self.filestatus.on_compile(Err(()))
        }
    }
}

pub(crate) struct FileStatus {
    pub(crate) currentfile: Option<PathBuf>,
    pub(crate) displayname: String,
    pub(crate) unsaved: bool,
    pub(crate) uncompiled: bool,
    pub(crate) compilefail: bool,
}
impl Default for FileStatus {
    fn default() -> Self {
        Self {
            currentfile: None,
            displayname: "Untitled".into(),
            unsaved: false,
            uncompiled: true,
            compilefail: false,
        }
    }
}
impl FileStatus {
    pub(crate) fn new_with_path(path: Option<PathBuf>) -> Self {
        let mut new = Self::default();
        new.currentfile = path;
        new.update_title();
        new
    }
    pub(crate) fn code_changed(&mut self) {
        self.unsaved = true;
        self.uncompiled = true;
        self.compilefail = false;
        self.update_title();
    }

    fn on_compile(&mut self, res: Result<(), ()>) {
        self.uncompiled = !matches!(res, Ok(_));
        self.compilefail = !matches!(res, Ok(_));
    }
    fn on_save(&mut self) {
        self.unsaved = false;
        self.update_title();
    }

    fn update_title(&mut self) {
        let filename: String = match &self.currentfile {
            Some(path) => path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap()
                .into(),
            _ => "Untitled".into(),
        };
        self.displayname = format!("{}{}", filename, if self.unsaved { "*" } else { "" });
    }
}
