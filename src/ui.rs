use std::io::{self, Write};

pub fn print_otp_with_progress(code: &str, rem: u64, quiet: bool) {
    if !quiet {
        let bar_width = 20;
        let safe_rem = rem.min(30) as usize;

        let filled = (safe_rem * bar_width) / 30;
        let empty = bar_width - filled;

        let bar = format!(
            "[{}{}] {}s remaining",
            "#".repeat(filled),
            ".".repeat(empty),
            safe_rem
        );

        // print to STDERR
        eprintln!("{}", bar);
    }

    // print the code to STDOUT
    println!("{}", code);

    let _ = io::stdout().flush();
}
