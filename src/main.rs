#[macro_use]
extern crate crossterm;

use crossterm::cursor;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use std::io::stdout;

use std::thread;
use std::time::{Duration, Instant};

use anyhow::anyhow;

fn main() -> anyhow::Result<()> {
    println!("Waiting for Wind Waker to start...");
    let dolphin = loop {
        if let Ok(p) = dolphin_memory::Dolphin::new() {
            // it takes a moment to initialize ram,
            // so let's just wait a second here before we check game version
            thread::sleep(Duration::from_secs(1));

            if !windwaker::gcm::is_supported_wind_waker(&p) {
                return Err(anyhow!("Unsupported game or version of TLoZ: The Wind Waker! This must be the NA GameCube release!"));
            }
            break p;
        }
    };

    let mut stdout = stdout();
    //going into raw mode
    enable_raw_mode()?;

    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    //clearing the screen, going to top left corner and printing welcoming message
    execute!(
        stdout,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print(r#"ctrl + q to exit, DPad-Left for BURST!!!"#)
    )?;

    let mut last_burst = Instant::now();
    //key detection
    loop {
        //going to top left corner
        execute!(stdout, cursor::MoveTo(0, 0))?;

        let input = windwaker::input::Inputs::default()
            .read(&dolphin)
            .unwrap_or_default();

        let can_burst = Instant::now().duration_since(last_burst).as_millis() > 300;

        if input.dpad_left_just_pressed && can_burst {
            last_burst = Instant::now();
            windwaker::player::Speed::default().write(1600.0, &dolphin)?;
        }

        // matching the key
        if poll(Duration::from_millis(1000 / 30))? {
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                }) => {
                    execute!(stdout, Clear(ClearType::All))?;
                    break;
                }
                _ => {}
            }
        }
    }

    //disabling raw mode
    disable_raw_mode()?;

    Ok(())
}
