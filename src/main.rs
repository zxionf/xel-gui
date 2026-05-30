mod debug;

fn main() {
    println!("[xel]xel >_<");
    debug::App::run().expect("[xel-test] 失败");
}
