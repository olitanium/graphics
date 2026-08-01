struct FakeStrLt<'a> {
    name: &'a str,
}

type FakeStrStatic = FakeStrLt<'static>;

#[test]
fn test() {
    let string = FakeStrStatic { name: "oliver" };

    let other_string = String::from("bad");
    // let bad = FakeStrStatic {
    // name: &other_string,
    // };
}
