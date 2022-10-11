## Requiem
WIP requirement engine for the Guild gate. Aimed to be generic over various
boolean logic and requirements.

Requirements are types that implement the `Requirement` trait which has a
single `check` function that retunrns a boolean value. Every type that
implements `Requirement` can be considered to be a terminal in a Binary
Decision Diagram or
[BDD](https://docs.rs/boolean_expression/latest/boolean_expression/struct.BDD.html).

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
```
where each element in the `requirements` vector can be deserialized into a type
that implements `Requirement` and they can all be evaluated to a `bool` by
performing the `check` function which is `async`, so multiple checks can be
performed at once. Once all requirements return with a boolean, the `BDD`
parsed from the `logic` field is evaluated on these terminal inputs. Note that
terminals identifiers in expression `logic` denote the respective requirement's
index in the `requirements` vector.
