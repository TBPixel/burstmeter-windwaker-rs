#[macro_use]
extern crate crossterm;

use anyhow::anyhow;
use crossterm::cursor;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use dolphin_memory::Dolphin;
use self_update::cargo_crate_version;

use std::io::stdout;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> anyhow::Result<()> {
    update()?;

    println!("Waiting for Wind Waker to start...");
    let dolphin = loop {
        if let Ok(p) = dolphin_memory::Dolphin::new() {
            // it takes a moment to initialize ram,
            // so let's just wait a couple seconds here before we check game version
            thread::sleep(Duration::from_secs(3));

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
        let mut mp = windwaker::player::Mp::default()
            .read(&dolphin)
            .unwrap_or_default();

        let can_burst =
            mp.current >= 2 && Instant::now().duration_since(last_burst).as_millis() > 200;

        if input.dpad_left_hold && can_burst {
            if let Ok(_) = charge_magic_cost(&mut mp, 2, &dolphin) {
                burst(1600.0, &dolphin);
            }
            last_burst = Instant::now();
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

// charge_magic_cost is a helper function that consumes the given amount of magic
// or returns an error.
fn charge_magic_cost(
    mp: &mut windwaker::player::Mp,
    amount: u8,
    d: &Dolphin,
) -> anyhow::Result<()> {
    if mp.current < amount {
        return Err(anyhow!("not enough magic!"));
    }

    if let Err(_) = mp.write_current(mp.current - amount, d) {
        return charge_magic_cost(mp, amount, d);
    }

    Ok(())
}

fn burst(amount: f32, d: &Dolphin) {
    if let Err(_) = windwaker::player::Speed::default().write(amount, d) {
        return burst(amount, d);
    }
}

fn update() -> anyhow::Result<()> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("TBPixel")
        .repo_name("burstmeter-windwaker-rs")
        .bin_name("github")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    println!("Update status: `{}`!", status.version());
    Ok(())
}
