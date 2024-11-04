#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

//use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use arduino_hal::{
    Delay,
    port::{
        Pin,
        mode::Output,
    },
    hal::port::Dynamic,
};

use panic_halt as _;

use ag_lcd::{ LcdDisplay, Lines };
use morse_codec::decoder::Decoder;

mod millis;
use millis::*;

struct CursorPosition(u8, u8);

const DEBOUNCE: u32 = 30;
const COLS: u8 = 16;
const MSG_MAX: usize = 32;

#[inline(never)]
fn calculate_cursor_positions(edit_pos: usize) -> CursorPosition {
    let y = edit_pos as u8 / COLS;
    let x = edit_pos as u8 - (y * COLS);

    CursorPosition(x, y)
}

#[inline(never)]
fn print_message(lcd: &mut LcdDisplay<Pin<Output, Dynamic>, Delay>, message: &str) {
    lcd.cursor_off();
    for (index, byte) in message.bytes().enumerate() {
        let tmp_pos = calculate_cursor_positions(index);
        lcd.set_position(tmp_pos.0, tmp_pos.1);
        arduino_hal::delay_ms(1);

        lcd.write(byte);
        arduino_hal::delay_ms(1);
    }
    lcd.cursor_on();
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    //let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    //LCD Display setup
    let lcd_delay = arduino_hal::Delay::new();
    let lcd_register_select = pins.d9.into_output().downgrade(); //Register select pin
    let lcd_enable = pins.d8.into_output().downgrade(); //Enable pin
    let lcd_data_4 = pins.d7.into_output().downgrade();
    let lcd_data_5 = pins.d6.into_output().downgrade();
    let lcd_data_6 = pins.d5.into_output().downgrade();
    let lcd_data_7 = pins.d4.into_output().downgrade();

    let mut lcd: LcdDisplay<_,_> = LcdDisplay::new(lcd_register_select, lcd_enable, lcd_delay)
        .with_half_bus(lcd_data_4, lcd_data_5, lcd_data_6, lcd_data_7)
        .with_lines(Lines::TwoLines)
        .with_reliable_init(10000)
        .build();

    lcd.cursor_on();
    lcd.blink_on();

    let mut cursor_pos = CursorPosition(0, 0);
    lcd.set_position(cursor_pos.0, cursor_pos.1);

    // BUTTONS
    let mut last_button_time: u32 = 0;

    let signal_input = pins.d13.into_pull_up_input();
    let mut can_signal = true;

    let print_input = pins.d12.into_pull_up_input();
    let mut can_print_message = true;

    let shift_left_input = pins.d11.into_pull_up_input();
    let mut can_shift_left = true;
    let shift_right_input = pins.d10.into_pull_up_input();
    let mut can_shift_right = true;

    // Morse decoder setup
    let mut decoder = Decoder::<MSG_MAX>::new()
        .with_reference_short_ms(100)
        .build();

    if !decoder.message.is_empty() {
        let message = decoder.message.as_str();
        print_message(&mut lcd, message);

        //ufmt::uwriteln!(&mut serial, "MESSAGE LENGTH: {}", message.len()).unwrap_infallible();
        //ufmt::uwriteln!(&mut serial, "MESSAGE: {}", message).unwrap_infallible();

        let edit_pos = decoder.message.get_edit_pos();
        cursor_pos = calculate_cursor_positions(edit_pos);
        lcd.set_position(cursor_pos.0, cursor_pos.1);

        //ufmt::uwriteln!(&mut serial, "Cursor position start: x {} y {} and edit pos: {}", cursor_pos.0, cursor_pos.1, edit_pos).unwrap_infallible();
    }

    let mut last_signal_time: u32 = 0;
    let mut last_space_time: u32 = 0;

    // Timer setup
    millis_init(dp.TC0);
    unsafe { avr_device::interrupt::enable() };

    loop {
        if signal_input.is_low() && can_signal {
            can_signal = false;
            last_signal_time = millis();

            if last_signal_time - last_button_time >= DEBOUNCE {
                //ufmt::uwriteln!(&mut serial, "KEY PRESSED: LAST SIGNAL TIME: {}, LAST SPACE TIME: {}", last_signal_time, last_space_time).unwrap_infallible();
                if last_space_time > 0 {
                    let diff = last_signal_time - last_space_time;

                    decoder.signal_event(diff as u16, false);

                    last_space_time = 0;
                }
            }
        } else if signal_input.is_high() && !can_signal {
            last_space_time = millis();
            last_button_time = last_space_time;

            //ufmt::uwriteln!(&mut serial, "\t\tKEY RELEASED: LAST SIGNAL TIME: {}, LAST SPACE TIME: {}", last_signal_time, last_space_time).unwrap_infallible();

            let diff = last_space_time - last_signal_time;

            decoder.signal_event(diff as u16, true);

            last_signal_time = 0;

            can_signal = true;
        }

        if print_input.is_low() && can_print_message {
            can_print_message = false;

            //ufmt::uwriteln!(&mut serial, "PRESSED LAST BUTTON TIME: {}, millis: {}", last_button_time, millis()).unwrap_infallible();
            if millis() - last_button_time >= DEBOUNCE {
                decoder.signal_event_end(false);

                let message = decoder.message.as_str();
                print_message(&mut lcd, message);

                //ufmt::uwriteln!(&mut serial, "MESSAGE LENGTH: {}", message.len()).unwrap_infallible();
                //ufmt::uwriteln!(&mut serial, "MESSAGE: {}", message).unwrap_infallible();

                // Restore cursor position to last edit position
                let edit_pos = decoder.message.get_edit_pos();
                cursor_pos = calculate_cursor_positions(edit_pos);
                lcd.set_position(cursor_pos.0, cursor_pos.1);

                arduino_hal::delay_ms(5);
                //ufmt::uwriteln!(&mut serial, "New cursor position input: x {} y {} and edit pos: {}", cursor_pos.0, cursor_pos.1, edit_pos).unwrap_infallible();
            }
        } else if print_input.is_high() && !can_print_message {
            last_button_time = millis();
            can_print_message = true;
        }

        if shift_left_input.is_low() && can_shift_left {
            can_shift_left = false;

            if millis() - last_button_time >= DEBOUNCE {
                decoder.message.shift_edit_left();
                let edit_pos = decoder.message.get_edit_pos();
                cursor_pos = calculate_cursor_positions(edit_pos);

                lcd.set_position(cursor_pos.0, cursor_pos.1);

                //ufmt::uwriteln!(&mut serial, "New cursor position left: x {} y {} and edit pos: {}", cursor_pos.0, cursor_pos.1, edit_pos).unwrap_infallible();
            }
        } else if shift_left_input.is_high() && !can_shift_left {
            last_button_time = millis();
            can_shift_left = true;
        }

        if shift_right_input.is_low() && can_shift_right {
            can_shift_right = false;

            if millis() - last_button_time >= DEBOUNCE {
                decoder.message.shift_edit_right();
                let edit_pos = decoder.message.get_edit_pos();
                cursor_pos = calculate_cursor_positions(edit_pos);

                lcd.set_position(cursor_pos.0, cursor_pos.1);

                //ufmt::uwriteln!(&mut serial, "New cursor position right: x {} y {} and edit pos: {}", cursor_pos.0, cursor_pos.1, edit_pos).unwrap_infallible();
            }
        } else if shift_right_input.is_high() && !can_shift_right {
            last_button_time = millis();
            can_shift_right = true;
        }

        arduino_hal::delay_ms(1);
    }
}
