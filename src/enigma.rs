use std::fs;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::io::{self, BufRead};

extern crate yaml_rust;
use yaml_rust::{Yaml, YamlLoader};

const ALPHA: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

static WHEELS: &'static[(&str, &str)] = &[
  ("IC", "DMTWSILRUYQNKFEJCAZBPGXOHV"),
  ("IIC", "HQZGPJTMOBLNCIFDYAWVEUSRKX"),
  ("IIIC", "UQNTLSZFMREHDPXKIBVYGJCWOA"),
  ("Reflector A", "EJMZALYXVBWFCRQUONTSPIKHGD")
];

struct Wheel {
  _name: String,
  wiring: String,
  step: usize,
  fixed: bool,
}

#[derive(Debug)]
struct WheelNotFound {
}

impl std::fmt::Display for Wheel {
   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
     write!(f, "{} at step {}", self._name, self.step)
   }
}

impl Wheel {
  pub fn new(name: &str, wiring: &str) -> Wheel {
    return Wheel { _name: name.to_string(), wiring: wiring.to_string(), step: 0, fixed: false } ;
  }

  pub fn by_name(name: &str) -> Result<Wheel, WheelNotFound> {
     for set in WHEELS {
       if set.0 == name {
         return Ok(Wheel::new(name, set.1));
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
        let new_index = Wheel::modulo(index as i32 + self.step as i32, self.wiring.len() as i32);
        return self.wiring.chars().nth(new_index as usize).unwrap();
      },
      None => panic!("Letter encoding missing!")
    }
  }

  pub fn send_left(&self, c: char) -> char {
    match self.wiring.find(c) {
      Some(index) => {
        let new_index = Wheel::modulo(index as i32 - self.step as i32, ALPHA.len() as i32);
        return ALPHA.chars().nth(new_index.abs() as usize).unwrap();
      },
      None => panic!("Letter encoding missing!")
    }
  }
}

struct Enigma {
  wheels: Vec<Wheel>,
  plugs_left: HashMap<char, char>,
  plugs_right: HashMap<char, char>,
}

impl Enigma {
  fn setup(&mut self, file_path: &str) {
      /* read the file */
      let settings = YamlLoader::load_from_str(&*fs::read_to_string(file_path).expect("to read settings")).expect("to read settings");
      /* load wheels */
      let doc = &settings[0];
      let wheels = &doc["wheels"];
      for w in wheels.as_vec().unwrap() {
        let wset = w.as_hash().unwrap();
        let Some(name) = wset.get(&Yaml::from_str("name")).unwrap().as_str() else { panic!("Name fnot found"); };
        let Some(pos) = wset.get(&Yaml::from_str("position")).unwrap().as_i64() else { panic!("Position not found"); } ;
        let mut wheel = Wheel::by_name(name).unwrap();
        wheel.step = pos as usize;
        self.wheels.push(wheel);
      }
      self.wheels.last_mut().unwrap().fixed = true;
      let plugs = &doc["plugs"];
      for p in plugs.as_hash().unwrap().iter() {
         let a = p.0.as_str().unwrap().chars().next().unwrap();
         let b = p.1.as_str().unwrap().chars().next().unwrap();
         self.plugs_left.insert(a, b);
         self.plugs_right.insert(b, a);
      }
   }

   fn new() -> Enigma {
     return Enigma { wheels: Vec::new(), plugs_left: HashMap::new(), plugs_right: HashMap::new() };
   }

   fn plug_left(&self, c: char) -> char {
      match self.plugs_left.get(&c) {
        Some(new_c) => return *new_c,
        None => return c
      }
   }

   fn plug_right(&self, c: char) -> char {
      match self.plugs_right.get(&c) {
        Some(new_c) => return *new_c,
        None => return c
      }
   }

   fn send(&mut self, mut c_in: char) -> char {
      /* go through wheels twice */
      let mut c = c_in;
      let mut wi: usize = 0;
      let mut dir = 0;
      /* plug board */
      c = self.plug_left(c);
      loop {
        c_in = c;
        let Some(wheel) = self.wheels.get_mut(wi) else { panic!("Wheel index {} missing", wi) };
        if dir == 0 {
           c = wheel.send_right(c_in);
        } else {
           c = wheel.send_left(c_in);
        }

        if dir == 0 {
          wi += 1;
        } else {
          if wi == 0 {
            break;
          }
          wi -= 1;
        }
        if wi >= self.wheels.len() {
          dir = 1;
          wi -= 2;
        }
      }
      /* plug it again */
      c = self.plug_right(c);

      /* step the wheels */
      let mut step_next = true;
      for wheel in &mut self.wheels {
        if wheel.fixed {
          continue;
        }
        if step_next {
          step_next = false;
          wheel.step += 1;
          if wheel.step >= wheel.wiring.len() {
            wheel.step = 0;
            step_next = true
          }
        }
      }

      /* All rotated, reset to 0 */
      if step_next {
        for wheel in &mut self.wheels {
          wheel.step = 0;
        }
      }

      return c
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
           None => result.push(c)
        }
     }
     return result;
   }
}

pub fn main() {
  let mut enigma = Enigma::new();
  let args: Vec<String> = env::args().collect();
  enigma.setup(&args[2]);
  let stdin =io::stdin();
  for line in stdin.lock().lines() {
    let cipher = enigma.encrypt(&line.unwrap());
    println!("{cipher}");
  }
}
