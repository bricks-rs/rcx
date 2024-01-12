use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

const DEVICE: &str = "/dev/usb/legousbtower0";

// const MSG: &[u8] = &[0x55, 0xff, 0x51, 0xae, 0x01, 0xfe, 0x52, 0xad];
const MSG: &[u8] = &[0x00, 0x30, !0x30, 0x30, !0x30];

fn main() {
    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(DEVICE)
        .unwrap();

    let mut f2 = f.try_clone().unwrap();

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        f.write_all(MSG).unwrap();
        println!("written");
    });

    loop {
        let mut buf = [0x00; 100];
        match f2.read(&mut buf) {
            Ok(len) => println!("Read {} bytes: {:02x?}", len, &buf[..len]),
            Err(e) => {
                dbg!(e);
            }
        }
    }
}
