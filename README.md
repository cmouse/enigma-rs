# Enigma

To learn a bit of Rust I wrote this Enigma encryption tool. It pretty much implements Enigma cipher, although I didn't
carefully verify compability with an actual machine.

Don't use for production or anything like that.

To use, 

```
cargo run -- enigma path/to/enigma.yaml
```

## Configuration

The tool accepts a simple YAML configuration. 

```yml
wheels:
  - name: Wheel Name
    position: Initial offset
  - name: Wheel Name
    position: Initial offset
  - name: Reflector A/B/C...
    position: Initial offset
plugs:
  A: C
  D: F
  ...
```

The last wheel should be a Reflector. Plugs must be unique mappings.

Unlike real enigma, this system will happily accept two or more wheels. The last wheel does not need to be
a reflector wheel, but it should be.

## Wheels

Currently following wheel names are supported:

I, II, III, IV, V, VI, VII, VII, VII, IC, IIC and IIIC.

Last wheel should be a reflector:

Reflector A, Reflector B, Reflector C, Reflector B Thin, Reflector C Thin.

## Usage

The tool reads input from stdin by lines, and writes to stdout. Only letters from A to Z are encrypted, anything else
is pass-through.

# References

[Wikipedia: Engima machine](https://en.wikipedia.org/wiki/Enigma_machine)
