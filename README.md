# PAML

Possibly A Markup Language. Probably not.

This is a rip-off of JSON and YAML. The implementation is in the [rust](/rust) folder (using serde).
Large chunks of it were copied from the serde documentation. Currently incomplete (floats, negative numers, and raw strings can't be parsed)

PAML has 6 built-in data types:

- Booleans: `true` and `false`
- Numbers
- `null` (may be unnecessary?)
- Strings (3 kinds)
    - Quoted (either `"foo"` or `'foo'`)
    - To end of line (`\`foo bar baz\n...` is the same as `"foo bar baz"\n...`)
        - These may be too much, might just remove them
    - Unquoted words (any sequence of non-whitespace characters that doesn't include `{}[]`)
 - Lists (space-separated): `[item1 item2 item3]`
 - Maps (also space-separated): `{ k1 v1 k2 v2 }`

TODO maybe treat all scalars as strings and let the deserializing thingy convert them to booleans or numbers?

Comments use `#`. Types can be specified using `~` (e.g. `~double 2`)

No idea how to formally specify a language so here's an example:

```
{
  people [
    {
      name Alice
      age 100
      "favorite command" `rm -rf / # This 'comment' is actually included in the string
      human true
    }
    {
      name Bob
      age 200
      "favorite command" ls
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
    edition "2021" # Quotes are necessary here to show it's a string, not a number
  }
  
  dependencies {
    serde { version "1.0" features [derive] }
    # todo possibly allow commas so ^ can be { version "1.0", features [derive] }, which is less confusing
  }
}
```
