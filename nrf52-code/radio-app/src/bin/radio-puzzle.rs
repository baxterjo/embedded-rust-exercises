#![deny(unused_must_use)]
#![no_main]
#![no_std]

use core::str;

use cortex_m_rt::entry;
use dk::ieee802154::{Channel, Packet};
use heapless::LinearMap;
use heapless::String;
// this imports `src/lib.rs`to retrieve our global logger + panicking-behavior
use radio_app as _;

const TEN_MS: u32 = 10_000;

#[entry]
fn main() -> ! {
    let board = dk::init().unwrap();
    let mut radio = board.radio;
    let mut timer = board.timer;

    // puzzle-fw uses channel 25 by default
    // NOTE if you ran `change-channel` then you may need to update the channel here
    radio.set_channel(Channel::_20); // <- must match the Dongle's listening channel

    let mut packet = Packet::new();

    // try one of these 3 options
    //let msg = b"";

    let mut map: LinearMap<u8, u8, 128> = LinearMap::new();
    let mut string: String<128> = String::new();

    // these 3 lines are equivalent
    //let msg: &[u8; 1] = b"B";
    // let msg: &[u8; 1] = &[b'A'];
    //let msg: &[u8; 1] = &[65];

    let msg = b"Hello?";

    for i in msg {
        let msg = &[i];
        defmt::println!(
            "sending: {}",
            str::from_utf8(msg).expect("msg was not valid UTF-8 data")
        );

        if let Ok(reply) = dk::send_recv(&mut packet, msg, &mut radio, &mut timer, TEN_MS) {
            let str = str::from_utf8(reply).expect("response was not valid UTF-8 data");
            defmt::println!("received: {}", str);

            let _ = map.insert(i, reply[0]);
            let _ = string.push_str(str);
        } else {
            defmt::error!("no response or response packet was corrupted");
        }
    }
    defmt::println!("map: {}", defmt::Debug2Format(&map));
    defmt::println!("string: {}", string.as_str());

    dk::exit()
}
