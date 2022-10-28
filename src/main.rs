use invaders::frame::{new_frame, Drawable};
use invaders::player::Player;
use invaders::{frame, render};
use rusty_audio::Audio;
use std::error::Error;
use std::{io, thread};
use std::sync::mpsc;
use crossterm::{terminal, ExecutableCommand};
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event::{self, KeyCode};
use std::time::{Duration, Instant};
use crossterm::event::Event;

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explode", "explode.wav");
    audio.add("lose", "lose.wav");
    audio.add("move", "move.wav");
    audio.add("pew", "pew.wav");
    audio.add("startup", "startup.wav");
    audio.add("win", "win.wav");
    audio.play("startup");

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?; // '?' means crash if there is an error
    stdout.execute(EnterAlternateScreen)?; // AlternateScreen: same as vim screen when enter it
    stdout.execute(Hide)?; // Hide cursor 

    // Render loop in a separate thread 
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    // Game Loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    'gameloop: loop {
        // Per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
               match key_event.code {
                KeyCode::Left => player.move_left(),
                KeyCode::Right => player.move_right(),
                KeyCode::Char(' ') | KeyCode::Enter => {
                    if player.shoot() {
                        audio.play("pew");
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    audio.play("lose");
                    break 'gameloop;
                }
                _ => {}
               } 
            }
        }

        // Updates
        player.update(delta);

        // Draw & render
        player.draw(&mut curr_frame);
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
