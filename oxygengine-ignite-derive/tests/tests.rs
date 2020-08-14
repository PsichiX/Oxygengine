use oxygengine_ignite_derive::Ignite;

#[derive(Ignite)]
#[ignite(namespace = "test")] // when omitted crate name is used.
struct Foo {
    #[ignite(ignore)]
    u: (),
    #[ignite(min = 0, max = 100)]
    a: i32,
    #[ignite(mapping = "math.Vec2")]
    b: (f32, f32),
    #[ignite(readonly)]
    c: [String; 2],
    #[ignite(default, another_custom_empty_tag)]
    d: std::collections::HashMap<String, Vec<Bar>>,
}

#[derive(Ignite)]
#[ignite(namespace = "test")]
enum Bar {
    A,
    B(#[ignite(ignore)] i32, f64),
    C {
        #[ignite(ignore)]
        a: i32,
        #[ignite(readonly)]
        b: f64,
    },
}
