# PAML

Possibly A Markup Language. Probably not.

This is a rip-off of JSON and YAML. The implementation is in the [rust](/rust) folder.
Large chunks of it were copied from the serde documentation.

No idea how to formally specify a language so here's some examples:

```
{
  key value
  foo [
    list and map entries are delimited by whitespace
  ]
  "null" null
  booleans [ true, false ]
  "and types" ~int 2
  backticks `for strings that continue to the end of the line
}
```
