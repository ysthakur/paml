# PAML

Possibly A Markup Language. Probably not.

This is a serialization format that is a rip-off of JSON and YAML. The implementation is in the [rust](/rust) folder.
Large chunks of it were copied from the serde documentation. Currently incomplete (floats, negative numers, and raw strings can't be parsed)

PAML has 6 built-in data types:

- Booleans: `true` and `false`
- Numbers
- `null` (may be unnecessary?)
- Strings

- Lists (space-separated, commas optional): `[item1 item2 item3]`
- Maps (also space-separated, commas optional): `{ k1 v1 k2 v2, k3 v3 }`

### Strings

Bare words/strings are string without quotes. They're any sequence of non-whitespace characters that don't include any special characters.

Strings can be quoted with an odd number of `"`, `'`, or <code>\`</code> characters (<code>\`</code> delimits a raw string where escapes don't do anything).

They can optionally be prefixed with either `unindent` or `singleLine` (no whitespace between the word and the quotes) (TODO actually implement this).

This string:
```
unindent">foo bar
          baz asdlfkjasdf
          jwosafasdfasdf"
```

is equivalent to this:
```
foo bar
baz asdlfkjasdf
jwosafasdfasdf
```

The `>` tells PAML which character index to strip whitespace until.

### Comments

All comments must contain valid tokens

Single line comments use `#`. A "single line" comment may contain a multiline string token. Just a quirk of how the parser works, I might change this later.

Multiline comments use `#[ ... ]#` (stolen from Nim).

## Example

No idea how to formally specify a language so here's an example:

```paml
{
  people [
    {
      name Alice
      age 100
      "favorite command" `rm -rf /`
      human true
    }
    {
      name Bob
      age 200
      "favorite command" `grep "\""`
      human false
    }
  ]
}
```

The [Cargo.toml](/rust/Cargo.toml) for this project would look something like this:

```paml
{
  package {
    name paml # Quotes are optional here
    version "0.1.0" # Would be recognized as a string rather than a number even without the quotes
    license MIT
    edition "2024" # Quotes are necessary here to show it's a string, not a number
  }

  dependencies {
    serde { version "1.0" features [derive] }
  }
}
```

## TODOs

- Allow keys like `foo.bar.baz` in dictionaries for nested objects (to make it more useful as a config file format)
- Byte strings
- Formatter
