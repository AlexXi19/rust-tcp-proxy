#![feature(test)]
extern crate test;

use tcp_proxy::proxy::*;
use test::Bencher;

#[bench]
fn benchmark_proxy_throughput(b: &mut Bencher) {
    // TODO: 
}
