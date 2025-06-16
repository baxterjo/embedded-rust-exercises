#![deny(unused_must_use)]
#![no_main]
#![no_std]

use core::str;

use cortex_m_rt::entry;
use dk::ieee802154::{Channel, Packet};
use heapless::LinearMap;
use heapless::Vec;
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

    // these 3 lines are equivalent
    //let msg: &[u8; 1] = b"B";
    // let msg: &[u8; 1] = &[b'A'];
    //let msg: &[u8; 1] = &[65];

    //let msg = b"Hello?";
    let mut empty = Packet::new();
    let Ok(secret) = dk::send_recv(&mut empty, &[], &mut radio, &mut timer, TEN_MS) else {
        defmt::error!("no response or response packet was corrupted");
        dk::exit();
    };

    for i in 0..128u8 {
        let msg = &[i];
        defmt::println!(
            "sending: {}",
            str::from_utf8(msg).expect("msg was not valid UTF-8 data")
        );

        if let Ok(reply) = dk::send_recv(&mut packet, msg, &mut radio, &mut timer, TEN_MS) {
            let str = str::from_utf8(reply).expect("response was not valid UTF-8 data");
            defmt::println!("received: {}", str);

            let _ = map.insert(reply[0], i);
        } else {
            defmt::error!("no response or response packet was corrupted");
        }
    }

    let mut answer: Vec<u8, 60> = Vec::new();
    for k in secret {
        defmt::println!("Checking {}", k);
        let a = map.get(k).expect("Map does not contain key");
        answer.push(*a).expect("Answer vec is full");
    }

    defmt::println!(
        "Answer is {}",
        str::from_utf8(&answer).expect("Answer is invalid UTF-8")
    );
    if let Ok(reply) = dk::send_recv(&mut packet, &answer[..], &mut radio, &mut timer, TEN_MS) {
        let str = str::from_utf8(reply).expect("response was not valid UTF-8 data");
        defmt::println!("solution was {}", str);
    } else {
        defmt::error!("no response or response packet was corrupted");
    }

    dk::exit()
}
