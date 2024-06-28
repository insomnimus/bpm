use std::{
	env,
	io::{
		stdout,
		Result,
		Write,
	},
	process,
	time::Instant,
};

use crossterm::{
	event::{
		self,
		Event,
		KeyCode,
		KeyEventKind,
		KeyModifiers,
		KeyboardEnhancementFlags,
		PopKeyboardEnhancementFlags,
		PushKeyboardEnhancementFlags,
	},
	terminal,
	ExecutableCommand,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

fn show_help() {
	println!(
		"\
{NAME} v{VERSION}
Tap keys to calculate beats per minute (BPM)

OPTIONS:
  -h, --help: Print usage help
  -V, --version: Print version information\
"
	);
}

fn run() -> Result<()> {
	let mut enhanced = cfg!(windows) || terminal::supports_keyboard_enhancement().unwrap_or(false);
	terminal::enable_raw_mode()?;
	if !cfg!(windows) && enhanced {
		enhanced = stdout()
			.execute(PushKeyboardEnhancementFlags(
				KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
			))
			.is_ok();
	}

	let close = || {
		if enhanced {
			let _ = stdout().execute(PopKeyboardEnhancementFlags);
		}
		terminal::disable_raw_mode()
	};

	stdout().write_all(
		b"\
Press any character key to tap
Timing is from the first tap to the last
Press esc to finish\n",
	)?;

	// Read once to start the timer.
	loop {
		if let Event::Key(k) = event::read()? {
			if enhanced && k.kind != KeyEventKind::Release {
				continue;
			}

			match k.code {
				KeyCode::Char('c' | 'C') if k.modifiers == KeyModifiers::CONTROL => return close(),
				KeyCode::Esc => return close(),
				KeyCode::Char(_) if k.modifiers.is_empty() => break,
				_ => (),
			}
		}
	}

	let start = Instant::now();
	let mut last = start;
	let mut count = 0_u32;

	loop {
		if let Event::Key(k) = event::read()? {
			if enhanced && k.kind != KeyEventKind::Release {
				continue;
			}

			match k.code {
				KeyCode::Char('c' | 'C') if k.modifiers == KeyModifiers::CONTROL => break,
				KeyCode::Esc => break,
				KeyCode::Char(_) if k.modifiers.is_empty() => {
					count += 1;
					last = Instant::now();
				}
				_ => (),
			}
		}
	}

	close()?;

	let micros = (last - start).as_micros();
	let minutes = micros as f64 / 60e6_f64;
	let bpm = count as f64 / minutes;
	println!("{bpm:.2} BPM");
	println!("{} presses in {:.1}s", count, micros as f64 / 1e6);
	Ok(())
}

fn main() {
	if let Some(a) = env::args_os().nth(1) {
		let exit_code = if &a == "-h" || &a == "--help" {
			show_help();
			0
		} else if &a == "-V" || &a == "--version" {
			println!("{NAME} v{VERSION}");
			0
		} else {
			eprintln!("error: unknown option {}", a.as_os_str().to_string_lossy());
			1
		};
		process::exit(exit_code);
	}

	if let Err(e) = run() {
		eprintln!("error: {e}");
		process::exit(1);
	}
}
