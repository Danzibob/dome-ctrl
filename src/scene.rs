use std::fs::File;
use std::io::{BufRead, BufReader};

/// Represents a single frame of the scene
/// N = Number of Nodes (rows)
/// M = Number of channels per Node (columns)
#[derive(Debug, Clone, Copy)]
pub struct Frame<const N: usize, const M: usize> {
    pub data: [[u8; M]; N],
}

impl<const N: usize, const M: usize> Frame<N, M> {
    pub fn new() -> Self {
        // Initialize a frame with zeros (all off)
        Self {
            data: [[0; M]; N],
        }
    }

    // Example: set the value of a specific LED and channel
    pub fn set_node(&mut self, led_idx: usize, values: [u8;M]) {
        if led_idx < N {
            self.data[led_idx] = values;
        } else {
            panic!("LED index out of bounds");
        }
    }
}

/// Represents a scene made up of multiple frames
#[derive(Debug)]
pub struct Scene<const N: usize, const M: usize> {
    pub frames: Vec<Frame<N, M>>,
    pub current_frame: usize,
}

impl<const N: usize, const M: usize> Scene<N, M> {
    /// Creates a new empty scene
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            current_frame: 0,
        }
    }

    pub fn new_from_file(filename: &str) -> Self {
        let file = File::open(filename).expect("Failed to open file");
        let reader = BufReader::new(file);
        let mut scene = Self::new();

        let mut current_frame = Frame::<N, M>::new();

        for line in reader.lines() {
            let line = line.expect("Failed to read line");
            // Parse the line and create a frame
            if line.starts_with("#"){
                continue;
            } else if line == "show" {
                scene.add_frame(current_frame);
                current_frame = Frame::<N, M>::new();
            } else {
                let mut values = line.split_whitespace();
                let index = values.next().expect("Index missing from line")
                                .parse().expect("Failed to parse index");
                let mut node = [0; M];
                for (i,value) in values.enumerate() {
                    node[i] = value.parse().expect("Failed to parse value");
                }
                current_frame.set_node(index, node);
                
            }
        }
        scene
    }

    /// Adds a frame to the scene
    pub fn add_frame(&mut self, frame: Frame<N, M>) {
        self.frames.push(frame);
    }

    /// Gets the total number of frames
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn next(&mut self) {
        if self.current_frame < self.frame_count() - 1 {
            self.current_frame += 1;
        }
    }

    pub fn prev(&mut self) {
        if self.current_frame > 0 {
            self.current_frame -= 1;
        }
    }

    pub fn get_node(&self, idx: usize) -> [u8; M] {
        self.frames[self.current_frame].data[idx]
    }
}
