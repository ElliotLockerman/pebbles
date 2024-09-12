
# Pebbles

Pebbles is a cli/repl programmer's calculator featuring
- Wrapping arithmetic
- Formatted output in decimal, hexadecimal, octal, and binary
- Fixed-size signed and unsigned types
- Bitwise operators


# Examples

Wrapping arithmetic:

```
$ pebbles --type=u8 '2 * 0x83'
6₁₀
        6₁₆
0000 0110₂
```

Octal, bitwise operators:

```
$ pebbles --type=u8 --base=oct '1 << (2 | 1)'
8₁₀
     1   0₈
00 001 000₂
```

Signed types, repl:

```
$ pebbles --type=i32                         
> 0o12 + -2 * 6
-2₁₀
   F    F    F    F    F    F    F    E₁₆
1111 1111 1111 1111 1111 1111 1111 1110₂
```


# Usage

- Options
    - `--base <base>`: base for output. One of {`hext`, `oct`}, default `hex`. Decimal and binary output are always printed. For signed types, decimal output is printed with a negative sign when appropriate; hex, oct, and binary output always reflects the bit pattern directly.
    - `--type <TYPE>`: one of {`u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`}, default `u32`. All values in the expression are of the selected type.
- Positional Arguments:
    - `[EXER]`: an expression to evaluate. If not provided, the repl is entered.

Literals can be decimal (no prefix), hexadecimal (`0x` prefix), or octal (`0o` prefix). A unary `-` gives the two's complement for both signed and unsigned types.

Pebbles operations generally tries to emulate machine primitives. For example, rather than being undefined behavior, shifts are mod the machine width:

```
$ pebbles --type=u8
 2 << 3
16₁₀
   1    0₁₆
0001 0000₂
> 2 << 11
16₁₀
   1    0₁₆
0001 0000₂
```

However, for practical reasons, full multiply/full divide (with double width product/dividend, respectively) are not currently provided. Divide and mod follow the C-style truncating (i.e., round-to-zero) convention, with mod having the same sign and the left-hand side.

Operators generally follow the traditional C model, although either `~` or `!` are allowed for bitwise negation. Boolean and comparison operators are not available.

Operator precedence (greatest to least):

- unary `-`, `!`, `~`
- `*`, `/`, `%`
- `+`, `-`
- `<<`, `>>`
- bitwise `&`
- bitwise `^`
- bitwise `|`


