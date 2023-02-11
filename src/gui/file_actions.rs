use super::GuiMode;
use crate::{emulator::CtrlMSG, TitoApp};
use rfd::FileDialog;
use std::env::current_dir;

impl TitoApp {
    pub fn file_new(&mut self) {
        self.editor.source_code = "".into();
        self.editor.source_path = None;
        self.editor.update_linecount();
        self.guimode = GuiMode::Editor;
    }

    pub fn file_open(&mut self) {
        self.editor.open_file(
            FileDialog::new()
                .add_filter("TTK Source files", &["k91"])
                .set_directory(&self.working_dir)
                .pick_file(),
        );
        self.guimode = GuiMode::Editor;
        self.working_dir = current_dir().unwrap();
    }

    pub fn file_save(&mut self) {
        self.editor.save_file(None);
    }

    pub fn file_saveas(&mut self) {
        self.editor.save_file(
            FileDialog::new()
                .add_filter("TTK Source files", &["k91"])
                .set_directory(&self.working_dir)
                .save_file(),
        );
        self.working_dir = current_dir().unwrap();
    }

    pub fn file_compile(&mut self) {
        self.current_prog = self.editor.compile();
        self.emu_tx
            .send(CtrlMSG::LoadProg(self.current_prog.clone()));
    }
}
