use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use ws28xx_n_channel_spi::{rpi_ws281x, LEDs};

// Number of nodes on the lightstage
const NUM_MODULES: usize = 143;
// Lightstage has 9 LED channels per node
const CHANNELS_PER_MODULE: usize = 9;
// Total LEDS = modules * channels
const NUM_LEDS: usize = NUM_MODULES * CHANNELS_PER_MODULE;

// useful constant to turn off LEDs
const OFF: [u8; 9] = [0, 0, 0, 0, 0, 0, 0, 0, 0];

const HELP: &str = "\
---=== Dome Control v1.0 ===---

Press the following keys to choose the lighting colour (uniform lighting):

Non-Polarized
    R - Red
    G - Green
    B - Blue
    W - Warm White
    N - Neutral White

Polarized (Cool White)
    C - Circular
    V - Vertical
    H - Horizontal
    D - Diagonal

Other Controls
    A - All lights
    O - Turn off all lights
    Q - Turn off lights and quit

    Up Arrow - Increase brightness
    Down Arrow - Decrease brightness\
";

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    // Create the linux SPI device adapter
    // Max brightness is limited via the adapter library here: -------------v
    let hw_adapter = rpi_ws281x::setup::<CHANNELS_PER_MODULE>(NUM_MODULES, 120).unwrap();
    // Create an LED strip with that adapter
    let mut strip: LEDs<NUM_LEDS, CHANNELS_PER_MODULE, _> = LEDs::new(hw_adapter);

    // Variable for uniform lighting color
    let mut color = OFF;
    let mut brightness = 200;
    let mut quit = false;

    // Detect keypress events
    for c in stdin.keys() {
        //clearing the screen and going to top left corner
        write!(
            stdout,
            "{}{}{}",
            termion::cursor::Goto(1, 1),
            termion::clear::All,
            HELP
        )
        .unwrap();
        stdout.flush().unwrap();

        // Set the strip color based on the key pressed
        match c.unwrap() {
            // Switch color or turn off
            Key::Char('r') => color = [brightness, 0, 0, 0, 0, 0, 0, 0, 0],
            Key::Char('g') => color = [0, brightness, 0, 0, 0, 0, 0, 0, 0],
            Key::Char('b') => color = [0, 0, brightness, 0, 0, 0, 0, 0, 0],
            Key::Char('c') => color = [0, 0, 0, brightness, 0, 0, 0, 0, 0],
            Key::Char('w') => color = [0, 0, 0, 0, brightness, 0, 0, 0, 0],
            Key::Char('n') => color = [0, 0, 0, 0, 0, brightness, 0, 0, 0],
            Key::Char('v') => color = [0, 0, 0, 0, 0, 0, brightness, 0, 0],
            Key::Char('h') => color = [0, 0, 0, 0, 0, 0, 0, brightness, 0],
            Key::Char('d') => color = [0, 0, 0, 0, 0, 0, 0, 0, brightness],
            Key::Char('a') => color = [brightness; 9],
            Key::Char('o') => color = OFF,
            // Change brightness
            Key::Up if brightness < 250 => brightness += 10,
            Key::Down if brightness > 10 => brightness -= 10,
            // Quit (also turns off)
            Key::Char('q') => {
                color = OFF;
                quit = true;
            }
            _ => (),
        }

        // Send updated colors to the strip adapter
        for i in 0..NUM_MODULES {
            strip.set_node(i, color);
        }
        // Have the adapter write these to the hardware
        strip.write().unwrap();

        // Quit after turning off LEDs
        if quit {
            break;
        }
    }
}
