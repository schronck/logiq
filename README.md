## requiem
`requiem` is a requirement engine built mainly for the Guild gate. Aimed to be generic over
various boolean logic and requirements.

The goal is to collect all requirements in the form of a `json` for example:
```
{
	"logic": "0 AND (1 OR 2 XOR (3 OR 1))",
	"requirements": [
		{ ... },
		{ ... },
		{ ... },
		{ ... }
	]
}
``` where each element in the `requirements` vector represents something that
can eventually be evaluated into a boolean. However, the above structure is
independent from the implementation of `requirement`. The only restriction is
that terminals (or leafs) in the logic expression should be represented by a
number. This number may denote the index of a requirement in a `requirements`
vector or it can be a key in a map that points to a specific requirement value.

### Usage

`requiem` parses the incoming `logic` data into a binary `LogicTree` where each
leaf represents a requirement terminal. Each terminal has a unique `TerminalId`
and holds a boolean value. Once the input logic is parsed, and the requirement
results are collected in a `HashMap<TerminalId, bool>`, then the `LogicTree`
root can be evaluated to a final boolean.

```
let tree = LogicTree::from_str("0 AND 1 OR ((0 NAND 2) OR 3)").unwrap();
// evaluate requirements
let mut evals = HashMap::new();
evals.insert(0, true);
evals.insert(1, false);
evals.insert(2, false);
evals.insert(3, true);

// boolean output of `true && false || (!(true && false) || true)`
assert!(tree.evaluate(&terminals).unwrap());
```

### Benchmarks

Benchmarked on an AMD Ryzen 3600, the following benchmarks times were measured (in $\mu$s):

| # of terminals | 10 | 100 | 1000 |
|-|-|-|
| parsing | 0.99 | 12.00 | 121.16 |
| evaluation | 0.16 | 1.95 | 21.44 |

Thus, even in a (imo highly unlikely) situation with 1000 requirements, parsing and evaluating
arbitrary logic takes less than 150 microseconds.
