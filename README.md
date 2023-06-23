# logiq

A logic operation descriptor language and evaluator engine inspired by
[requiem](https://github.com/agoraxyz/requiem)
and the
[Lisp programming language](<https://en.wikipedia.org/wiki/Lisp_(programming_language)>).

## Usage

`logiq` parses the `logic_str` into an
[AST](https://en.wikipedia.org/wiki/Abstract_syntax_tree)
and replaces the *terminal id*s with the boolean values from `thruths` at the
indices defined by the terminal ids.

There are three atomic types in logiq:

| name       | description                               |
| ---------- | ----------------------------------------- |
| TerminalId | thruth index                              |
| Gate       | logic gate (and, or, not, nand, nor, xor) |
| List       | a vector of atomic types                  |

These atomic types are used just like in a Lisp language, a `TerminalId` is a
valid expression by itself, but if we try to evaluate a `Gate` it will return an
error message. We can only use logic gates in lists, surrounded by terminal ids.
There is only one unary operator, which is the `NOT` operator, all the other
gates are binary operators.
While Lisp uses prefix operators, logiq uses infix operators which are more
intuitive (`NOT` is a prefix operator in logiq, too).

### Example

```rs
let thruths = [true, false, true];

// true => true
let logic_str = "2";
assert!(!eval(logic_str, &thruths));

// (not true) => false
let logic_str = "(NOT 0)";
assert!(!eval(logic_str, &thruths));

// ((true AND false) OR true) => true
let logic_str = "((0 AND 1) OR 2)";
assert!(eval(logic_str, &thruths));
```

## Benchmarks

Benchmarked on a 2020 M1 Macbook Air:

| # of terminals  | 10   | 100   | 1000   |
| --------------- | ---- | ----- | ------ |
| parsing [μs]    | 2.53 | 22.12 | 222.22 |
| evaluation [μs] | 2.59 | 23.91 | 242.72 |
