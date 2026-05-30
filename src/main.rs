mod debug;

fn main() {
    println!("[xel-test] xel >_<");
    debug::App::run().expect("[xel-test] 失败");
}
