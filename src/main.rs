use rust_hdl::prelude::*;
use std::{env, fs};

const CLOCK_SPEED_HZ: u64 = 10_000;

#[derive(LogicBlock)]
pub struct LedController {
    pub clock: Signal<In, Clock>,
    pub buttons: Signal<In, Bits<4>>,
    pub leds: Signal<Out, Bits<4>>,
}

impl Logic for LedController {
    #[hdl_gen]
    fn update(&mut self) {
        self.leds.next = self.buttons.val();
    }
}

impl Default for LedController {
    fn default() -> Self {
        LedController {
            clock: Default::default(),
            buttons: Default::default(),
            leds: Default::default(),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("args = {:?}", &args[1..]);
    let verilog = args[1] == "verilog";
    let mut uut = LedController::default();
    if verilog {
        uut.connect_all();
        let vlog = generate_verilog(&uut);
        println!("{vlog}");
        fs::write("verilog.v", vlog);
    } else if args[1] == "test" {
        let test_cases = (0..16)
            .map(|n| ((n as u8).to_bits::<4>(), (n as u8).to_bits::<4>()))
            .collect::<Vec<_>>();
        let mut sim = simple_sim!(LedController, clock, CLOCK_SPEED_HZ, ep, {
            let mut x = ep.init()?;
            for test_case in &test_cases {
                x.buttons.next = test_case.0;
                wait_clock_cycle!(ep, clock, x);
                println!(
                    "buttons = {:?}, leds = {:?} (check = {:?})",
                    test_case.0,
                    x.leds.val(),
                    test_case.1
                );
                sim_assert_eq!(ep, x.leds.val(), test_case.1, x);
            }
            ep.done(x)
        });
        sim.run(LedController::default().into(), sim_time::ONE_SEC)
            .unwrap();
    } else {
        let mut sim = simple_sim!(LedController, clock, CLOCK_SPEED_HZ, ep, {
            let mut x = ep.init()?;
            wait_clock_cycle!(ep, clock, x, 4 * CLOCK_SPEED_HZ);
            ep.done(x)
        });
        sim.run_to_file(Box::new(uut), 5 * sim_time::ONE_SEC, "ledcontroller.vcd")
            .unwrap();
    }
}
