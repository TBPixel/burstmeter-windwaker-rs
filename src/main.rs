#[macro_use]
extern crate crossterm;

use anyhow::anyhow;
use crossterm::cursor;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use dolphin_memory::Dolphin;
use self_update::cargo_crate_version;
use thiserror::Error;

use std::io::{stdout, Stdout};
use std::thread;
use std::time::{Duration, Instant};

// main checks for updates and then runs app
fn main() -> anyhow::Result<()> {
    // check for updates
    update()?;

    app()
}

// app includes a basic runtime, but doesn't check for updates
fn app() -> anyhow::Result<()> {
    // block and wait for wind waker to start
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
    // going into raw mode
    enable_raw_mode()?;

    //clearing the screen, going to top left corner and printing welcoming message
    execute!(
        stdout,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print(r#"ctrl + q to exit, DPad-Left for BURST!!!"#)
    )?;

    // Run the app, block, and on "emulation not running" error re-run from the beginning
    if let Err(err) = runtime(&mut stdout, dolphin) {
        match err {
            RuntimeError::EmulationNotRunning => {
                disable_raw_mode()?;
                return app();
            }
        }
    };

    disable_raw_mode()?;

    Ok(())
}

#[derive(Error, Debug)]
enum RuntimeError {
    #[error("emulation not running")]
    EmulationNotRunning,
}

// runtime blocks and loops over memory. A RuntimeError can be returned and optionally handled.
fn runtime(stdout: &mut Stdout, dolphin: Dolphin) -> anyhow::Result<(), RuntimeError> {
    // keeping track of the last_burst
    let mut last_burst = Instant::now();
    loop {
        // check to see if Dolphin was closed
        if !dolphin.is_emulation_running() {
            let _ = execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0));
            return Err(RuntimeError::EmulationNotRunning);
        }

        // return cursor to top left of console
        if let Err(_) = execute!(stdout, cursor::MoveTo(0, 0)) {
            continue;
        }

        // watch inputs and magic levels on loop
        let input = windwaker::input::Inputs::default()
            .read(&dolphin)
            .unwrap_or_default();
        let mut mp = windwaker::player::Mp::default()
            .read(&dolphin)
            .unwrap_or_default();

        // simple check to determine if we can burst
        let can_burst =
            mp.current >= 2 && Instant::now().duration_since(last_burst).as_millis() > 200;

        // trigger burst of speed and update last_burst timer
        if input.dpad_left_hold && can_burst {
            if let Ok(_) = charge_magic_cost(&mut mp, 2, &dolphin) {
                burst(1600.0, &dolphin);
                last_burst = Instant::now();
            }
        }

        // watch for exit input eg. CTRL + Q
        if !poll(Duration::from_millis(1000 / 30)).unwrap_or(false) {
            continue;
        }

        // read input events, and on error we'll just loop again because it's cheap
        let event = match read() {
            Ok(event) => event,
            Err(_) => continue,
        };
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            }) => {
                // ignore the error because it's honestly fine if we don't clear here.
                let _ = execute!(stdout, Clear(ClearType::All));
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

// charge_magic_cost is a helper function that consumes
// the given amount of magic or returns an error.
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

// burst of speed is set to the player. If the burst fails to apply, it tries again automatically.
fn burst(amount: f32, d: &Dolphin) {
    if let Err(_) = windwaker::player::Speed::default().write(amount, d) {
        return burst(amount, d);
    }
}

// update checks for a new release on GitHub, and if so it downloads.
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
