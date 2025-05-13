#![no_std]
#![no_main]

use teensy4_bsp as bsp;
use teensy4_panic as _;

use bsp::board;

use imxrt_log::log;
use teensy4_pins::{t41, common, Config, configure, PullKeeper, OpenDrain};
use bsp::hal::{
    flexpwm::{Channel, Output, PairOperation, Prescaler, Submodule, FULL_RELOAD_VALUE_REGISTER},
    iomuxc::consts::*,
};
use bsp::ral;


use bsp::rt;
use embedded_io::{Read, Write};

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

#[bsp::rt::entry]
fn main() -> ! {
    let board::Resources {
        //mut ccm, // clock control module
        //mut ccm_analog,
        flexpwm4: (mut pwm4, (_, mut sm1, _, _)),
        pins,
        mut gpio1,
        lpuart2,
        ..
    } = board::t41(board::instances());

    // pin 23 = gpio_ad_b1_09 → flexpwm4 pwma01 (module 4, sm 1, channel A)
    let pwm_pin = pins.p23;
    let out_a = Output::new_a(pwm_pin); // sets pin mux to alt‑2 for you

    // float pin 27, todo fix
    let float_pin = pins.p27;
    gpio1.input(float_pin);

    const PERIOD: i16 = 18;

    // ── submodule setup ────────────────────────────
    sm1.set_prescaler(Prescaler::Prescaler1);                // /1
    sm1.set_pair_operation(PairOperation::Independent);

    // counter runs 0 → PERIOD‑1
    sm1.set_initial_count(&mut pwm4, 0);
    sm1.set_value(FULL_RELOAD_VALUE_REGISTER, PERIOD - 1);

    // 50 % duty: toggle at half‑period
    out_a.set_turn_on(&sm1, 0);
    out_a.set_turn_off(&sm1, PERIOD / 2);

    // arm the output
    out_a.set_output_enable(&mut pwm4, true);
    sm1.set_load_ok(&mut pwm4); // copy buffered regs
    sm1.set_running(&mut pwm4, true); // GO
    // now pin 23 spews ~8.33 mhz square wave
    // this works! - confirmed on scope

    let mut ccm = unsafe { ral::ccm::CCM::instance() };
    let mut ccm_analog = unsafe { ral::ccm_analog::CCM_ANALOG::instance() };

    bsp::hal::ccm::analog::pll3::restart(&mut ccm_analog);
    bsp::hal::ccm::clock_gate::usb().set(&mut ccm, bsp::hal::ccm::clock_gate::ON);
    cortex_m::peripheral::NVIC::pend(interrupt::USB_OTG1);
    unsafe { cortex_m::peripheral::NVIC::unmask(interrupt::USB_OTG1) };

    loop {
        ::log::info!("running");
    }

}

