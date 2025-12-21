use crate::Value;

pub fn print(val: &Value) -> String {
  let mut buf = String::new();
  print_impl(val, &mut buf);
  buf
}

fn print_impl(val: &Value, buf: &mut String) {
  match val {
    Value::Bool { val, .. } => {
      buf.push_str(if *val { "true" } else { "false" });
    }
    Value::Num { val, .. } => {
      buf.push_str(&val.integer_part);
      if let Some(dec) = &val.decimal_part {
        buf.push('.');
        buf.push_str(dec);
      }
      if let Some(exp) = &val.exponent {
        buf.push('e');
        buf.push_str(exp);
      }
    }
    Value::Str { val, .. } => {
      buf.push('"');
      for c in val.chars() {
        match c {
          '\\' => buf.push_str("\\\\"),
          '\n' => buf.push_str("\\n"),
          '\r' => buf.push_str("\\r"),
          '\t' => buf.push_str("\\t"),
          '"' => buf.push_str("\\\""),
          _ => buf.push(c),
        }
      }
      buf.push('"');
    }
    Value::List { val, .. } => {
      buf.push('[');
      for item in val {
        print_impl(item, buf);
        // TODO no comma after last item
        buf.push(',');
      }
      buf.push(']');
    }
    Value::Map { val, .. } => {
      buf.push('{');
      for (key, val) in val {
        print_impl(key, buf);
        buf.push(' ');
        print_impl(val, buf);
        // TODO no comma after last item
        buf.push(',');
      }
      buf.push('}');
    }
  }
}
