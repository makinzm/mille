use crate::usecase::greet;
use crate::infrastructure::printer;

pub fn run() {
    let user = greet::hello();
    printer::print(&user);
}
