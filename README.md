Morse Code Decoder for Arduino UNO
=========

Morse code decoder embedded project for the _Arduino Uno_. It uses four buttons
to enter morse signals and create a message. Message is shown on a 16x2 LCD
display (HITACHI HD4478).

## Libraries used
1. [avr-hal](https://github.com/Rahix/avr-hal) as AVR hardware abstraction layer to compile code to AVR architecture
and use it's API's to program the chip.

2. [morse-codec](https://github.com/burumdev/morse-codec) as morse decoder library that forms text messages from
morse signals on the fly.

3. [ag-lcd](https://github.com/mjhouse/ag-lcd) as LCD display driver.

## Build Instructions
1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).

2. Run `cargo build` to build the firmware.

3. Run `cargo run` to flash the firmware to a connected board.  If `ravedude`
   fails to detect your board, check its documentation at
   <https://crates.io/crates/ravedude>.

4. `ravedude` will open a console session after flashing where you can interact
   with the UART console of your board. You can use this to debug the code.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme
[`ravedude`]: https://crates.io/crates/ravedude

## License
Licensed under

 - MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

## Project in Action
<img src="https://raw.githubusercontent.com/burumdev/morse-avr-demo/refs/heads/master/morse-avr-decoder.jpg" alt="morse decoder for arduino" />
