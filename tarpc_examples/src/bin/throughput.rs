extern crate rand;

#[macro_use] extern crate tarpc;

use rand::Rng;
use std::time;
use std::net;
use std::thread;
use std::io::{Read, Write};

fn gen_vec(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut vec: Vec<u8> = Vec::with_capacity(size);
    for _ in 0..size {
        vec.push(rng.gen());
    }
    vec
}

service! {
    rpc read(size: u32) -> Vec<u8>;
}

struct Server;

impl Service for Server {
    fn read(&self, size: u32) -> Vec<u8> {
        gen_vec(size as usize)
    }
}

const CHUNK_SIZE: u32 = 1 << 10;

fn bench_tarpc(target: u64) {
    let handle = Server.spawn("0.0.0.0:0").unwrap();
    let client = Client::new(handle.dialer()).unwrap();
    let start = time::Instant::now();
    let mut nread = 0;
    while nread < target {
        client.read(CHUNK_SIZE).unwrap();
        nread += CHUNK_SIZE as u64;
    }
    let duration = time::Instant::now() - start;
    println!("TARPC: {}MB/s",  (target / (1024 * 1024)) as u64 / duration.as_secs());
}

fn bench_tcp(target: u64) {
    let l = net::TcpListener::bind("0.0.0.0:0").unwrap();
    let addr = l.local_addr().unwrap();
    thread::spawn(move || {
        let (mut stream, _) = l.accept() .unwrap();
        let mut vec = gen_vec(CHUNK_SIZE as usize);
        while let Ok(_) = stream.write_all(&vec[..]) {
            vec = gen_vec(CHUNK_SIZE as usize);
        }
    });
    let mut stream = net::TcpStream::connect(&addr).unwrap();
    let mut buf = vec![0; CHUNK_SIZE as usize];
    let start = time::Instant::now();
    let mut nread = 0;
    while nread < target {
        stream.read_exact(&mut buf[..]).unwrap();
        nread += CHUNK_SIZE as u64;
    }
    let duration = time::Instant::now() - start;
    if duration.as_secs() == 0 {
        println!("TCP:   {:?}",  duration);
    } else {
        println!("TCP:   {}MB/s",  (target / (1024 * 1024)) as u64 / duration.as_secs());
    }
}

fn main() {
    bench_tarpc(10 << 20);
    bench_tcp(256 << 20);
}
