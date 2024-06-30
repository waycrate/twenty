mod session_lock;

fn main() {
    let _ = session_lock::lock();
}
