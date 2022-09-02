# sprintf library

A sprintf like library to help with formatting strings at runtime in rust.

It was originally written to help with a IoT / home automation gatway that is yet closed source and works with the [metrics_evaluation library](https://github.com/Dirk007/metrics_evaluation.git) which is also used in that project to evaluate variables at runtime.

## Example

Run

```bash
cargo run --example simple
```

See soures [here](examples/simple/)

## What can it do?

Print variables formatted to a string at runtime. That's it.
The syntax is lent by the [C function](https://cplusplus.com/reference/cstdio/printf/) but with _much less_ functionality and placeholders.

These placeholders are currently supported:

- %s (string)
- %d (decimal)
- %f (float)
- %x (hexadecimal lowercase)
- %X (hexadecimal uppercase)
- %v (just Display - which is lent by golang)

All numbers can be formatted with a much simplified C version format.

Use `%06d` to prepend fill the decimal with zeroes up to 6 characters. `123` will become `000123`. Same for hex.

Use `%04.02f` to format `1.2` to `0001.20`.

## Is this considered "feature complete"?

No. This library is just at a state where I can basically use it for the above mentioned project.

## Need more?

Implement it and just PR. I am happy for every help.

## License

[MIT](./LICENSE)
