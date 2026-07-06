#![windows_subsystem = "windows"]

fn main() -> iced::Result {
    // https://www.reddit.com/r/learnrust/comments/jaqfcx/windows_print_to_hidden_console_window/
    #[cfg(target_os = "windows")]
    {
        use winapi::um::wincon::{ATTACH_PARENT_PROCESS, AttachConsole};
        // SAFETY: AttachConsole is safe to call even if there is no parent console;
        // it returns 0 on failure which we discard intentionally.
        unsafe {
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }

    rdict_iced::run()
}
