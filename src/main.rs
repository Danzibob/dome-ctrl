use clap::Parser;
use std::io::{stdin, stdout, Write};
use std::path::Path;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use ws28xx_n_channel_spi::{rpi_ws281x, LEDs};

mod scene;
use scene::Scene;

// Number of nodes on the lightstage
const NUM_MODULES: usize = 143;
// Lightstage has 9 LED channels per node
const CHANNELS_PER_MODULE: usize = 9;
// Total LEDS = modules * channels
const NUM_LEDS: usize = NUM_MODULES * CHANNELS_PER_MODULE;

// useful constant to turn off LEDs
const OFF: [u8; 9] = [0; CHANNELS_PER_MODULE];

// TODO: all white, no RGB

const BASIC_HELP: &str = "\
Press to choose the lighting colour (uniform):  \r
Non-Polarized           Polarized (Cool White)  \r
    R - Red                 C - Circular        \r
    G - Green               V - Vertical        \r
    B - Blue                H - Horizontal      \r
    W - Warm White          D - Diagonal        \r
    N - Neutral White       P - All Polarized   \r
                                                \r
Other Controls                                  \r
    A - All lights                              \r
    O - Turn off all lights                     \r
    Esc - Turn off lights and quit              \r
    Up Arrow - Increase brightness              \r
    Down Arrow - Decrease brightness            \r
\n\n\r";

const SCENE_HELP: &str = "\
Controls for Scene Player:                      \r
    Right Arrow - Next scene                    \r
    Left Arrow - Previous scene                 \r
    Esc - Quit                                  \r
\n\n\r";

fn match_mode(k: char, brightness: u8) -> Option<[u8; CHANNELS_PER_MODULE]> {
    let arr = match k {
        'r' => [brightness, 0, 0, 0, 0, 0, 0, 0, 0],
        'g' => [0, brightness, 0, 0, 0, 0, 0, 0, 0],
        'b' => [0, 0, brightness, 0, 0, 0, 0, 0, 0],
        'c' => [0, 0, 0, brightness, 0, 0, 0, 0, 0],
        'w' => [0, 0, 0, 0, brightness, 0, 0, 0, 0],
        'n' => [0, 0, 0, 0, 0, brightness, 0, 0, 0],
        'v' => [0, 0, 0, 0, 0, 0, brightness, 0, 0],
        'h' => [0, 0, 0, 0, 0, 0, 0, brightness, 0],
        'd' => [0, 0, 0, 0, 0, 0, 0, 0, brightness],
        'p' => [
            0, 0, 0, brightness, 0, 0, brightness, brightness, brightness,
        ],
        'a' => [brightness; 9],
        'o' => OFF,
        _ => return None,
    };
    Some(arr)
}

/// Program arguments struct
#[derive(Parser, Debug)]
#[command(
    name = "My Program",
    version = "1.0",
    about = "Runs interactive mode or scene player mode"
)]
struct Cli {
    /// Optional file input for the scene player
    #[arg(short, long, value_name = "FILE")]
    file: Option<String>,
}

fn interactive_basic() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", BASIC_HELP).unwrap();
    stdout.flush().unwrap();

    // Create the linux SPI device adapter
    // Max brightness is limited via the adapter library here: -------------v
    let hw_adapter = rpi_ws281x::setup::<CHANNELS_PER_MODULE>(NUM_MODULES, 255).unwrap();
    // Create an LED strip with that adapter
    let mut strip: LEDs<NUM_LEDS, CHANNELS_PER_MODULE, _> = LEDs::new(hw_adapter);

    // Variable for uniform lighting color
    let mut pixel = OFF;
    let mut color = 'o';
    let mut brightness = 200;
    let mut quit = false;

    // Detect keypress events
    for c in stdin.keys() {
        // Key event unwrapping
        match c.unwrap() {
            // If keypress was a letter AND it matches a color,
            // Set that color for both the pixel and mode
            Key::Char(k) => {
                if let Some(px) = match_mode(k, brightness) {
                    pixel = px;
                    color = k;
                }
            }
            // If up or down arrow was pressed, change brightness
            Key::Up if brightness < 250 => {
                brightness += 10;
                pixel = match_mode(color, brightness).unwrap();
            }
            Key::Down if brightness > 10 => {
                brightness -= 10;
                pixel = match_mode(color, brightness).unwrap();
            }
            // Quit (also turns off)
            Key::Esc | Key::Ctrl('c') => {
                pixel = OFF;
                quit = true;
            }
            _ => {}
        }

        //clearing the screen and going to top left corner
        write!(
            stdout,
            "{}{}{}Brightness: {}\tColor: {}\n\r",
            termion::cursor::Left(100),
            termion::cursor::Up(1),
            termion::clear::CurrentLine,
            brightness,
            color
        )
        .unwrap();
        stdout.flush().unwrap();

        // Send updated colors to the strip adapter
        for i in 0..NUM_MODULES {
            strip.set_node(i, pixel);
        }
        // Have the adapter write these to the hardware
        strip.write().unwrap();

        // Quit after turning off LEDs
        if quit {
            break;
        }
    }
}

fn scene_player(file_path: &str) {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", SCENE_HELP).unwrap();
    stdout.flush().unwrap();

    // Create the linux SPI device adapter
    // Max brightness is limited via the adapter library here: -------------v
    let hw_adapter = rpi_ws281x::setup::<CHANNELS_PER_MODULE>(NUM_MODULES, 255).unwrap();
    // Create an LED strip with that adapter
    let mut strip: LEDs<NUM_LEDS, CHANNELS_PER_MODULE, _> = LEDs::new(hw_adapter);

    let mut scene: Scene<NUM_MODULES, CHANNELS_PER_MODULE> = Scene::new_from_file(file_path);

    // Send updated colors to the strip adapter
    for i in 0..NUM_MODULES {
        strip.set_node(i, scene.get_node(i));
    }
    // Have the adapter write these to the hardware
    strip.write().unwrap();

    let mut quit = false;
    // Detect keypress events
    for c in stdin.keys() {
        // Key event unwrapping
        match c.unwrap() {
            Key::Right => {scene.next();}
            Key::Left  => {scene.prev();}
            // Quit (also turns off)
            Key::Esc | Key::Ctrl('c') => {
                quit = true;
            }
            _ => {}
        }

        //clearing the screen and going to top left corner
        write!(
            stdout,
            "{}{}{}Frame {} of {}\n\r",
            termion::cursor::Left(100),
            termion::cursor::Up(1),
            termion::clear::CurrentLine,
            scene.current_frame + 1,
            scene.frame_count()
        )
        .unwrap();
        stdout.flush().unwrap();

        // Send updated colors to the strip adapter
        for i in 0..NUM_MODULES {
            strip.set_node(i, scene.get_node(i));
        }
        // Have the adapter write these to the hardware
        strip.write().unwrap();

        // Quit after turning off LEDs
        if quit {
            strip.clear().unwrap();
            break;
        }
    }
}

fn main() {
    // Parse the command-line arguments into the Cli struct
    let args = Cli::parse();

    // Check if the file argument is provided
    if let Some(file) = args.file {
        if Path::new(&file).exists() {
            scene_player(&file);
        } else {
            println!("Error: File '{}' does not exist.", file);
        }
    } else {
        interactive_basic();
    }
}