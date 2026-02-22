fn main() {
    #[derive(Debug)]
    enum Foo { Bar }
    println!("\"{foo:?}\"", foo = Foo::Bar);
}
