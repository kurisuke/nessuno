mod bus;
mod cpu;

use bus::dummy::DummyBus;
use cpu::Cpu;

fn main() {
    let mut cpu = Cpu::new(Box::new(DummyBus::new()));

    println!("Hello, world!");
}
