use shmallocator::GlobalPSMAllocator;
use std::collections::HashMap;

#[global_allocator]
static PSMALLOCATOR: GlobalPSMAllocator = GlobalPSMAllocator;

fn main() {
    let mut map = HashMap::new();
    map.insert(0, String::from("hello"));
    map.remove(&0);
}
