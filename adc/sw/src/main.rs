#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;

use bsp::board;

use imxrt_log::log;
use teensy4_pins::{t41, common, Config, configure, PullKeeper, OpenDrain};
use bsp::hal::{
    ccm::{self, clock_gate, perclk_clk},
    timer::BlockingPit,
    flexpwm::{Channel, Output, PairOperation, Prescaler, Submodule, FULL_RELOAD_VALUE_REGISTER},
    iomuxc::consts::*,
};
use bsp::ral;

use bsp::rt;
use embedded_io::{Read, Write};
use embedded_hal::digital::InputPin;
use cortex_m::prelude::_embedded_hal_blocking_spi_Transfer;

mod regs;

const FRAME_LEN:usize = 15;
fn initialize_logger() -> Option<imxrt_log::Poller> {
    // logging
    let usb_instances = bsp::hal::usbd::Instances {
        usb: unsafe { ral::usb::USB1::instance() },
        usbnc: unsafe { ral::usbnc::USBNC1::instance() },
        usbphy: unsafe { ral::usbphy::USBPHY1::instance() },
    };
    let poller = log::usbd(usb_instances, imxrt_log::Interrupts::Enabled).ok()?;
    Some(poller)
}

use ral::interrupt;
#[bsp::rt::interrupt]
fn USB_OTG1() {
    static mut POLLER: Option<imxrt_log::Poller> = None;
    if let Some(poller) = POLLER.as_mut() {
        poller.poll();
    } else {
        let poller = initialize_logger().unwrap();
        *POLLER = Some(poller);
        // Since we enabled interrupts, this interrupt
        // handler will be called for USB traffic and timer
        // events. These are handled by poll().
    }
}

fn extract_data(words: &[u16]) -> [i32; 8] {
    assert_eq!(words.len(), 15, "i will *end* you if this isn’t 15");

    let mut bytes = [0u8; 30];
    for (chunk, &word) in bytes.chunks_exact_mut(2).zip(words) {
        chunk[0] = (word >> 8) as u8;
        chunk[1] = word as u8;
    }

    let mut out = [0i32; 8];
    for (i, slot) in out.iter_mut().enumerate() {
        let bit_offset = 24 + i * 24;
        let byte_offset = bit_offset / 8;
        let bit_shift = bit_offset % 8;

        let chunk = u32::from_be_bytes([
            bytes[byte_offset],
            bytes[byte_offset + 1],
            bytes[byte_offset + 2],
            bytes[byte_offset + 3],
        ]);

        let val = (chunk >> (8 - bit_shift)) & 0x00FF_FFFF;

        *slot = if val & 0x0080_0000 != 0 {
            (val | 0xFF00_0000) as i32
        } else {
            val as i32
        };
    }

    out
}



#[bsp::rt::entry]
fn main() -> ! {
    let board::Resources {
        mut ccm, // clock control module
        mut ccm_analog,
        flexpwm4: (mut pwm4, (_, mut sm1, _, _)),
        pins,
        mut gpio1,
        mut gpio2,
        mut gpio4,
        lpuart2,
        mut lpspi4,
        ..
    } = board::t41(board::instances());

    /// DELAY CONFIG
    // Before touching the PERCLK clock roots, turn off all downstream clock gates.
    clock_gate::PERCLK_CLOCK_GATES.iter().for_each(|loc| loc.set(&mut ccm, clock_gate::OFF));

    // Configure PERCLK to match this frequency:
    const PERCLK_CLK_FREQUENCY_HZ: u32 = ccm::XTAL_OSCILLATOR_HZ / PERCLK_CLK_DIVIDER;
    const PERCLK_CLK_DIVIDER: u32 = 24;
    perclk_clk::set_selection(&mut ccm, perclk_clk::Selection::Oscillator);
    perclk_clk::set_divider(&mut ccm, PERCLK_CLK_DIVIDER);

    // Turn on the PIT clock gate.
    clock_gate::pit().set(&mut ccm, clock_gate::ON);

    // There's no other divider, so the PIT frequency is the root
    // clock frequency.
    const PIT_FREQUENCY_HZ: u32 = PERCLK_CLK_FREQUENCY_HZ;

    let pit = unsafe { ral::pit::PIT::instance() };
    let (pit0, _, _, _) = bsp::hal::pit::new(pit);

    let mut blocking = BlockingPit::<0, PIT_FREQUENCY_HZ>::from_pit(pit0);


    blocking.block_ms(1000);
    // pin 23 = gpio_ad_b1_09 → flexpwm4 pwma01 (module 4, sm 1, channel A)
    let pwm_pin = pins.p23;
    let out_a = Output::new_a(pwm_pin); // sets pin mux to alt‑2 for you

    // float pin 27, todo fix
    let float_pin = pins.p27;
    gpio1.input(float_pin);

    // set SYNC/RESET pin high
    let sync_pin = pins.p2;
    let mut sync_pin_out = gpio4.output(sync_pin);
    sync_pin_out.set();

    const PERIOD: i16 = 18;

    // ── submodule setup ────────────────────────────
    sm1.set_prescaler(Prescaler::Prescaler1);                // /1
    sm1.set_pair_operation(PairOperation::Independent);

    sm1.set_initial_count(&mut pwm4, 0);
    sm1.set_value(FULL_RELOAD_VALUE_REGISTER, PERIOD - 1);

    out_a.set_turn_on(&sm1, 0);
    out_a.set_turn_off(&sm1, PERIOD / 2);

    out_a.set_output_enable(&mut pwm4, true);
    sm1.set_load_ok(&mut pwm4); // copy buffered regs
    sm1.set_running(&mut pwm4, true); // GO
    // now pin 23 spews ~8.33 mhz square wave
    // this works! - confirmed on scope
    // im a bit worried I may be misunderstanding how they count the period though

    // set up logger
    // later we'll send some data over this stream, inefficient but w/e
    bsp::hal::ccm::analog::pll3::restart(&mut ccm_analog);
    bsp::hal::ccm::clock_gate::usb().set(&mut ccm, bsp::hal::ccm::clock_gate::ON);
    cortex_m::peripheral::NVIC::pend(interrupt::USB_OTG1);
    unsafe { cortex_m::peripheral::NVIC::unmask(interrupt::USB_OTG1) };

    let mut lpspi4 = board::lpspi(
        lpspi4,
        board::LpspiPins {
            sdo: pins.p11,
            sdi: pins.p12,
            sck: pins.p13,
            pcs0: pins.p10,
        },
        1_000_000 // 1 MHz?
    );

    lpspi4.set_mode(bsp::hal::lpspi::MODE_1);


    let mut drdy = gpio2.input(pins.p9);
    // ADS131 docs want 10 24-bit words
    // but I can't set the word size to 24 here (there is no u24 type)
    // words in SPI are not delimited
    // so let's read 15 24-bit words and reshuffle them later???
    let mut adc_pkt : [u16; FRAME_LEN] = [0; FRAME_LEN];
    let mut drdy_prev = false;

    // spin lock until drdy goes low for the first time..
    while !drdy.is_low().unwrap() {}
    // then read 2 frames to clear the FIFO buffer
    // see datasheet section:
    // 8.5.1.9.1 Collecting Data for the First Time or After a Pause in Data Collection
    for i in 0..2 {
        for j in 0..adc_pkt.len() {
            adc_pkt[i] = 0u16;
        }
        lpspi4.transfer(&mut adc_pkt).unwrap();
    }
    // now we know there will be exactly 1 new frame for each DRDY edge

    loop {
        if !drdy_prev && drdy.is_low().unwrap() {
            drdy_prev = true;
            lpspi4.transfer(&mut adc_pkt).unwrap();
            let status = regs::Status::from_word(adc_pkt[0]);
            ::log::info!("{:?}", status);
            let data = extract_data(&adc_pkt);
            //::log::info!("{:016b} {:016b} {:016b} {:016b} {:016b} {:016b} {:016b} {:016b} {:016b} {:016b} {:016b} {:016b} {:016b} {:016b} {:016b}", adc_pkt[0], adc_pkt[1], adc_pkt[2], adc_pkt[3], adc_pkt[4], adc_pkt[5], adc_pkt[6], adc_pkt[7], adc_pkt[8], adc_pkt[9], adc_pkt[10], adc_pkt[11], adc_pkt[12], adc_pkt[13], adc_pkt[14]);
            ::log::info!("{}", data[0]);
            for i in 0..adc_pkt.len() {
                adc_pkt[i] = 0u16;
            };

        }
        if drdy.is_high().unwrap() {
            drdy_prev = false;
        }
    }

}

