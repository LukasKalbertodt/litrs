use procmacro_example::{concat, repeat};

const FOO: &str = concat!(r#"Hello "# '🦊' "\nHere is a friend: \u{1F427}");
// const FOO: &str = concat!(3.14);

const BAR: &str = repeat!(3 * "నా పిల్లి లావుగా ఉంది");
const BAZ: &str = repeat!(0b101 * "🦀");


fn main() {
    println!("{}", FOO);
    println!("{}", BAR);
    println!("{}", BAZ);
}
