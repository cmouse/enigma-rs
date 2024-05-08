use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, BufRead};

extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

const ALPHA: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/* name, wirings, notch(es), fixed */
static WHEELS: &'static [(&str, &str, &str, bool)] = &[
    ("IC", "DMTWSILRUYQNKFEJCAZBPGXOHV", "Q", false),
    ("IIC", "HQZGPJTMOBLNCIFDYAWVEUSRKX", "E", false),
    ("IIIC", "UQNTLSZFMREHDPXKIBVYGJCWOA", "V", false),
    ("I", "JGDQOXUSCAMIFRVTPNEWKBLZYH", "Q", false),
    ("II", "NTZPSFBOKMWRCJDIVLAEYUXHGQ", "E", false),
    ("III", "JVIUBHTCDYAKEQZPOSGXNRMWFL", "V", false),
    ("IV", "ESOVPZJAYQUIRHXLNFTGKDCMWB", "J", false),
    ("V", "VZBRGITYUPSDNHLXAWMJQOFECK", "Z", false),
    ("VI", "JPGVOUMFYQBENHZRDKASXLICTW", "ZM", false),
    ("VII", "NZJHGRCXMYSWBOUFAIVLPEKQDT", "ZM", false),
    ("VIII", "FKQHTLXOCBJSPDZRAMEWNIUYGV", "ZM", false),
    ("Reflector A", "EJMZALYXVBWFCRQUONTSPIKHGD", "", true),
    ("Reflector B", "YRUHQSLDPXNGOKMIEBFZCWVJAT", "", true),
    ("Reflector C", "FVPJIAOYEDRZXWGCTKUQSBNMHL", "", true),
    ("Reflector B Thin", "ENKQAUYWJICOPBLMDXZVFTHRGS", "", true),
    ("Reflector C Thin", "RDOBJNTKVEHMLFCWZAXGYIPSUQ", "", true),
];

struct Wheel {
    _name: String,
    wiring: String,
    step: usize,
    notch: String,
    fixed: bool,
}

#[derive(Debug)]
struct WheelNotFound {}

impl std::fmt::Display for Wheel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{:02}", self._name, self.step)
    }
}

impl Wheel {
    pub fn new(name: &str, wiring: &str, notch: &str, fixed: bool) -> Wheel {
        Wheel {
            _name: name.to_string(),
            wiring: wiring.to_string(),
            step: 0,
            notch: notch.to_string(),
            fixed: fixed,
        }
    }

    pub fn by_name(name: &str) -> Result<Wheel, WheelNotFound> {
        for set in WHEELS {
            if set.0 == name {
                return Ok(Wheel::new(name, set.1, set.2, set.3));
            }
        }
        Err(WheelNotFound {})
    }

    fn modulo(n: i32, m: i32) -> i32 {
        ((n % m) + m) % m
    }

    pub fn send_right(&self, c: char) -> char {
        match ALPHA.find(c) {
            Some(index) => {
                let new_index =
                    Wheel::modulo(index as i32 + self.step as i32, self.wiring.len() as i32);
                return self.wiring.chars().nth(new_index as usize).unwrap();
            }
            None => panic!("Letter encoding missing!"),
        }
    }

    pub fn send_left(&self, c: char) -> char {
        match self.wiring.find(c) {
            Some(index) => {
                let new_index = Wheel::modulo(index as i32 - self.step as i32, ALPHA.len() as i32);
                return ALPHA.chars().nth(new_index.abs() as usize).unwrap();
            }
            None => panic!("Letter encoding missing!"),
        }
    }
}

struct Enigma {
    wheels: Vec<Wheel>,
    plugboard: HashMap<char, char>,
}

impl Enigma {
    fn setup(&mut self, file_path: &str) {
        /* read the file */
        let settings =
            YamlLoader::load_from_str(&*fs::read_to_string(file_path).expect("to read settings"))
                .expect("to read settings");
        /* load wheels */
        let doc = &settings[0];
        let wheels = &doc["wheels"];
        for w in wheels.as_vec().unwrap() {
            let wset = w.as_hash().unwrap();
            let Some(name) = wset.get(&Yaml::from_str("name")).unwrap().as_str() else {
                panic!("Name fnot found");
            };
            let Some(pos) = wset.get(&Yaml::from_str("position")).unwrap().as_i64() else {
                panic!("Position not found");
            };
            let mut wheel = Wheel::by_name(name).unwrap();
            wheel.step = pos as usize;
            self.wheels.push(wheel);
        }
        let plugs = &doc["plugs"];
        self.setup_plugboard(plugs)
    }

    fn new() -> Enigma {
        Enigma {
            wheels: Vec::new(),
            plugboard: HashMap::new(),
        }
    }

    fn setup_plugboard(&mut self, plugs: &Yaml) {
        // initialize with standard plugs
        for c in ALPHA.chars() {
            self.plugboard.insert(c, c);
        }

        match plugs.as_hash() {
            Some(plugs) => {
                for p in plugs {
                    let a = p.0.as_str().unwrap().chars().next().unwrap();
                    let b = p.1.as_str().unwrap().chars().next().unwrap();
                    *self.plugboard.get_mut(&a).unwrap() = b;
                    *self.plugboard.get_mut(&b).unwrap() = a; // Ensures bidirectional mapping
                }
            }
            None => {}
        }
    }

    fn apply_plugboard(&self, c: char) -> char {
        *self.plugboard.get(&c).unwrap_or(&c)
    }

    fn print_wheels(&self) {
        let mut bomb = String::new();
        for wheel in &self.wheels {
            if !wheel.fixed {
              bomb.push(ALPHA.chars().nth(wheel.step).unwrap());
            }
        }
        println!("Wheel position: {bomb}");
    }

    fn step_wheels(&mut self) {
        let mut step_next = true; // Rightmost rotor steps
        let nwheels = self.wheels.len();
        for i in 0..nwheels {
            {
              let wheel = &mut self.wheels[i];
              if wheel.fixed {
                continue;
              } // Skip fixed wheels like the reflector

              if step_next {
                  wheel.step = (wheel.step + 1) % ALPHA.len();
                  step_next = wheel.notch.contains(ALPHA.chars().nth(wheel.step).unwrap());
              }
            // Handle double-stepping anomaly
            if i < nwheels - 1
                && step_next
                && wheel.notch.contains(ALPHA.chars().nth(wheel.step).unwrap())
            {
                self.wheels[i + 1].step = (self.wheels[i + 1].step + 1) % ALPHA.len();
            }
            }
        }
    }

    fn send(&mut self, mut c_in: char) -> char {
        let mut wi: usize = 0;
        let mut dir = 0;

        /* Wheel stepping happens before cipher step */
        self.step_wheels();
        /* go through plug board */
        let mut c = self.apply_plugboard(c_in);
        //println!("plugboard, initial: {c_in} -> {c}");
        /* go through each loop */
        loop {
            c_in = c;
            let Some(wheel) = self.wheels.get_mut(wi) else {
                panic!("Wheel index {} missing", wi)
            };

            if dir == 0 {
                /* going from stator -> reflector */
                c = wheel.send_right(c_in);
            } else {
                /* going from reflector -> stator */
                c = wheel.send_left(c_in);
            }
            //println!("wheel {wi:02}-{:02}: {c_in} -> {c}", wheel.step);
            /* Choose next wheel */
            if dir == 0 {
                wi += 1;
            } else {
                if wi == 0 {
                    break;
                }
                wi -= 1;
            }

            /* Reached reflector, need to use the one wheel before reflector */
            if wi >= self.wheels.len() {
                dir = 1;
                wi -= 2;
            }
        }

        /* Once more through plugboard */
        c_in = c;
        c = self.apply_plugboard(c_in);
        //println!("plugboard, final: {c_in} -> {c}");
        c
    }

    fn reset(&mut self) {
        for wheel in &mut self.wheels {
            wheel.step = 0;
        }
    }

    fn encrypt(&mut self, plain: &str) -> String {
        let mut result = String::from("");
        for c in plain.to_uppercase().chars() {
            match ALPHA.find(c) {
                Some(_index) => result.push(self.send(c)),
                None => result.push(c),
            }
        }
        result
    }
}

pub fn main() {
    let mut enigma = Enigma::new();
    let args: Vec<String> = env::args().collect();
    enigma.setup(&args[2]);
    enigma.print_wheels();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let cipher = enigma.encrypt(&line.unwrap());
        println!("{cipher}");
    }
}
