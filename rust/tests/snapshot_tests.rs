use rstest::rstest;

#[rstest]
#[case("simple_value", "true")]
#[case("simple_list", r#"[foo 1.2 true false 'bar' "baz"]"#)]
fn test_parse(#[case] name: &str, #[case] code: &str) {
  let mut res = String::new();

  match paml::tokenize::tokenize(code) {
    Err(e) => {
      res.push_str(&format!("Error: {e:?}"));
    }
    Ok(tokens) => {
      for token in tokens {
        res.push_str(&format!("{token:?}\n"));
      }
    }
  }
  res.push_str("---\n");

  match paml::parse_lossless::parse(code.to_string()) {
    Err(e) => {
      res.push_str(&format!("Error: {e:?}"));
    }
    Ok(parse_res) => {
      res.push_str(&format!("Before: {:#?}\n", parse_res.before));
      res.push_str(&format!("Tree: {:#?}\n", parse_res.tree));
      res.push_str(&format!("After: {:#?}\n", parse_res.after));
      res.push_str("Validation errors: [\n");
      for err in parse_res.validation_errors {
        res.push_str(&format!("{err:?}\n"));
      }
      res.push_str("]\n")
    }
  }
  res.push_str("---\n");

  insta::assert_snapshot!(name, res);
}
