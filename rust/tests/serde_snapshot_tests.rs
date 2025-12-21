use std::collections::HashMap;

use serde::Serialize;

#[derive(Serialize)]
struct Struct<T, U, V, W> {
  unit: (),
  bool: bool,
  int: usize,
  double: f64,
  foo: T,
  bar: U,
  baz: V,
  foobar: W,
}

#[derive(Serialize)]
struct UnitStruct;

#[derive(Serialize)]
struct TupleStruct<A, B>(A, B);

#[derive(Serialize)]
enum Enum {
  StructVariant {
    a: usize,
  },
  UnitVariant,
  TupleVariant(String),
}

#[test]
fn test_big_serialize() {
  let mut map = HashMap::new();
  map.insert("foo", None);
  map.insert("bar", Some(123));
  let val = Struct {
    unit: (),
    bool: false,
    int: 123,
    double: 123.45,
    foo: UnitStruct,
    bar: TupleStruct(123.45, true),
    baz: vec![
      Enum::StructVariant { a: 123 },
      Enum::UnitVariant,
      Enum::TupleVariant("foo\\\nbar\t\"asdf'k;sdf".to_string()),
    ],
    foobar: map,
  };

  insta::assert_snapshot!("big_serialize", paml::serde::to_string(&val).unwrap());
}
