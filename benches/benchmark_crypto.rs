#![feature(test)]
extern crate test;

use tcp_proxy::proxy::crypto;
use test::Bencher;

#[bench]
fn benchmark_crypto(b: &mut Bencher) {
    let data = vec![0u8; 1024 * 1024];
    b.iter(|| {
        let _ = crypto::encrypt(data.clone());
    });
    b.iter(|| {
        let _ = crypto::decrypt(data.clone());
    });
}
