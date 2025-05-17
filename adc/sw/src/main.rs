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
    gpt::ClockSource,
};
use bsp::ral;

use bsp::rt;
use embedded_io::{Read, Write};
use embedded_hal::digital::InputPin;
use cortex_m::prelude::_embedded_hal_blocking_spi_Transfer;

mod regs;

const FRAME_LEN:usize = 30;
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

// ads131 uses 24 bit "words"
// 10 of those to a frame = 240 bits aka 30 u8s
// this discards the first and last 24 bits (status and CRC)
// then converts the remaining 8 into i32s (smallest type that fits 24 bits)
fn chop_bits(bytes: &[u8; 30]) -> [i32; 8] {
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


// "pc frames" are the frames that go to your pc
// as opposed to the frame we're receiving from the ADC
#[inline(always)]
fn mk_pc_frame(status: u8, micros: u32, data: &[i32; 8]) -> [u8; 38] {
    let mut buf = [0u8; 38];

    buf[0] = 0xA5; // sync byte
    buf[1] = status;

    buf[2..6].copy_from_slice(&micros.to_le_bytes());

    for (i, &val) in data.iter().enumerate() {
        buf[6 + i * 4..6 + (i + 1) * 4].copy_from_slice(&val.to_le_bytes());
    }

    buf
}

use arrayvec::ArrayVec;
use data_encoding::BASE64;
fn base64encode_frame<'a>(frame: &[u8], buf: &'a mut [u8]) -> &'a str {
    //let mut buf = ArrayVec::<u8, 128>::new();
    //// I mean frames are always the same size
    //// but you know
    BASE64.encode_mut_str(&frame, buf)
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
        mut gpt1,
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
    let (pit0, mut pit1, _, _) = bsp::hal::pit::new(pit);

    pit1.enable();
    let mut blocking = BlockingPit::<0, PIT_FREQUENCY_HZ>::from_pit(pit0);


    // uhh I'm leaving the delay config in
    // but we don't currently need a delay per se
    // blocking.block_ms(1000);

    /// RTC config



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
    let mut adc_pkt : [u8; FRAME_LEN] = [0; FRAME_LEN];
    let mut drdy_prev = false;

    // spin lock until drdy goes low for the first time..
    while !drdy.is_low().unwrap() {}
    // then read 2 frames to clear the FIFO buffer
    // see datasheet section:
    // 8.5.1.9.1 Collecting Data for the First Time or After a Pause in Data Collection
    for i in 0..2 {
        for j in 0..adc_pkt.len() {
            adc_pkt[i] = 0u8;
        }
        lpspi4.transfer(&mut adc_pkt).unwrap();
    }
    // now we know there will be exactly 1 new frame for each DRDY edge

    let mut outbuf = ArrayVec::<u8, 128>::new();
    let outbuflen = BASE64.encode_len(38);
    // super dumb but it's not even in the main loop so who cares tbh
    for i in 0..outbuflen {
        outbuf.push(0);
    }
    loop {
        if !drdy_prev && drdy.is_low().unwrap() {
            // pit1 counts down not up, so we invert to get timer value
            let micros: u32 = !pit1.current_timer_value();
            drdy_prev = true;
            lpspi4.transfer(&mut adc_pkt).unwrap();
            let status = regs::Status::from_bytes(&adc_pkt);
            let data = chop_bits(&adc_pkt);
            let frame = mk_pc_frame(0, micros, &data);
            let enc = base64encode_frame(&frame, outbuf.as_mut_slice());
            ::log::info!("{}", enc);
            for i in 0..adc_pkt.len() {
                adc_pkt[i] = 0u8;
            };

        }
        if drdy.is_high().unwrap() {
            drdy_prev = false;
        }
    }

}

