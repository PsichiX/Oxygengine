#[derive(oxygengine_ignite_derive::Ignite)]
#[ignite(namespace = test)]
struct Foo {
    u: (),
    a: i32,
    b: (i32, f64),
    // #[ignite(ignore)]
    c: [String; 2],
    d: std::collections::HashMap<String, Vec<Bar>>,
}

#[derive(oxygengine_ignite_derive::Ignite)]
enum Bar {
    A,
    B(i32, f64),
    C { a: i32, b: f64 },
}

#[test]
fn works() {
    // let foo = Foo {
    //     a: 42,
    //     b: 4.2,
    //     c: "42".to_owned(),
    //     d: Bar::B(1),
    // };
}
