# schermz

A CLI tool to create a schema from a JSON file.

## Installation

This tool is written in Rust, so you'll need to install the [Rust toolchain](https://www.rust-lang.org/tools/install) to build it.

```bash
cargo install schermz
```

## Usage

```bash
A tool to generate a schema for a given JSON file.

Usage: schermz [OPTIONS] --file <FILE>

Options:
  -f, --file <FILE>             Path to the JSON file
  -m, --merge-objects           Whether to merge object types into one
      --enum-threshold <N>      Emit a "values" enum for scalar fields with
                                at most N distinct observed values [default: 30]
  -h, --help                    Print help
  -V, --version                 Print version
```

## The `-m` argument

When this argument is passed to schermz, all objects for the same key will be merged into one, meaning, if a key can have multiple different object shapes, they will not be listed separately. This is useful when you want to get a general idea of the data, or you trust that the data is consistent.

Here's a simple example:

`sample.json`

```json
[
  {
    "info": {
      "name": "Martin",
      "age": 30
    }
  },
  {
    "info": {
      "name": "Paul"
    }
  }
]
```

### Without `-m` (default)

```bash
schermz -f ./sample.json

{
  "info": {
    "types": [
      {
        "age": {
          "types": [
            "NUMBER"
          ]
        },
        "name": {
          "types": [
            "STRING(6)"
          ]
        }
      },
      {
        "name": {
          "types": [
            "STRING(4)"
          ]
        }
      }
    ]
  }
}
```

### With `-m`

```bash
schermz -m -f ./sample.json

{
  "info": {
    "types": [
      {
        "age": {
          "optional": true,
          "types": [
            "NUMBER"
          ]
        },
        "name": {
          "types": [
            "STRING(4, 6)"
          ]
        }
      }
    ]
  }
}

```

Note `age` is annotated `"optional": true` — it was present on Martin but
not on Paul. See [Optional fields](#optional-fields) below.

## Optional fields

For every key, schermz reports whether it appeared in 100% of the observed
parent objects of its enclosing shape. When a key is *not* always present, it
gets `"optional": true`. Always-present keys carry no annotation, which keeps
the output tight.

This matters for codegen: it's the difference between `field: T` and
`field?: T` in TypeScript, or `z.string()` and `z.string().optional()` in Zod.

The denominator depends on context:

- For top-level keys, it's the number of objects in the input array (or 1 for
  a single-object input — schermz can't infer optionality from a sample size
  of 1, so nothing is ever marked optional in that case).
- For nested object keys, it's the number of parents that had this key set to
  an object of this shape.
- For object keys inside arrays, it's the total number of object elements
  observed across all parent arrays.

Without `-m`, distinct object shapes for the same key are listed separately.
Within any one variant, all objects by definition share the same keys, so
`optional` won't appear. Use `-m` to flatten variants and reveal which keys
are sometimes-present in the merged shape.

A key whose value is `null` still counts as "present" for this signal —
`{ "x": null }` is different from `{}`. The `null` shows up inside `types`,
not in the optionality flag.

## Enum values

When a scalar field has a small number of distinct observed values, schermz
emits them as a sorted `"values"` list alongside `"types"`. This lets
downstream codegen turn `z.string()` into `z.enum(["applied", "in_force"])`,
which is enforceable, surfaces in error messages, and gives compile-time
autocomplete.

```json
{
  "status": {
    "types": ["STRING(6, 8)"],
    "values": ["applied", "in_force", "opened", "revoked"]
  }
}
```

The threshold is configurable with `--enum-threshold N` (default `30`). Pass
`0` to disable the annotation entirely.

Rules:

- Only scalar fields. A key whose `types` mixes a scalar with an object,
  array, or boolean variant gets no `"values"`.
- Single non-null scalar variant only. `STRING + NULL` qualifies (the null is
  the absence of a value, not a competing scalar). `STRING + NUMBER` does
  not — too ambiguous to enumerate as one list.
- Booleans are skipped — trivially enumerable from the type alone, and the
  noise isn't worth it.
- Numbers are emitted only if every observed value is an integer. A single
  float in the set disqualifies it (an open enum of floats isn't useful).
- Strings longer than 200 characters disqualify the set, even if cardinality
  is below the threshold (JSON blobs, base64, free-text descriptions).

Strings sort lexicographically. Numbers sort numerically.

## Output

String values are analyzed based on their possible lengths.

- `STRING(0, 10)` - This field is a string with a minimum length of 0 (`""`) and a maximum length of 10.
- `STRING(5)` - This field is a string with a length of 5.

## Example

`sample.json`

```json
[
  {
    "name": "Sherlock Holmes",
    "title": "",
    "age": 34,
    "personal_data": {
      "gender": "male",
      "marital_status": "single"
    },
    "address": {
      "street": "10 Downing Street",
      "city": "London",
      "zip": "12345",
      "country_code": "UK"
    },
    "phones": ["+44 1234567", "+44 2345678", 12311, { "mobile": "+44 3456789" }]
  },
  {
    "name": "Tony Soprano",
    "title": "",
    "age": 39,
    "personal_data": {
      "gender": "male",
      "marital_status": "married"
    },
    "address": {
      "street": "14 Aspen Drive",
      "city": "Caldwell",
      "zip": "NJ 07006",
      "country": "USA",
      "state": "New Jersey",
      "country_code": "US"
    },
    "phones": [
      "+1 1234567",
      "+1 2345678",
      "+1 11111111111",
      "+1 301234566",
      11224234,
      { "mobile": "+1 3456789" }
    ]
  },
  {
    "name": "Angela Merkel",
    "title": "",
    "age": 65,
    "personal_data": {
      "gender": "female",
      "marital_status": "married"
    },
    "address": {
      "street": "Gr. Weg 3",
      "city": "Potsdam",
      "zip": "14467",
      "country": "Germany",
      "state": "Brandenburg"
    },
    "phones": [
      "+49 1234222567",
      "+49 2343231678",
      "+49 1111131111111",
      "+49 301212334566",
      9999222,
      { "mobile": "+49 343156789", "fax": "+49 343156780" }
    ]
  },
  {
    "name": "Jane Doe",
    "title": "Dr.",
    "age": "73",
    "personal_data": {
      "gender": "female"
    },
    "address": null,
    "phones": null
  }
]
```

```bash
schermz -f ./sample.json

{
  "address": {
    "types": [
      "NULL",
      {
        "city": {
          "types": ["STRING(6)"]
        },
        "country_code": {
          "types": ["STRING(2)"]
        },
        "street": {
          "types": ["STRING(17)"]
        },
        "zip": {
          "types": ["STRING(5)"]
        }
      },
      {
        "city": {
          "types": ["STRING(8)"]
        },
        "country": {
          "types": ["STRING(3)"]
        },
        "country_code": {
          "types": ["STRING(2)"]
        },
        "state": {
          "types": ["STRING(10)"]
        },
        "street": {
          "types": ["STRING(14)"]
        },
        "zip": {
          "types": ["STRING(8)"]
        }
      },
      {
        "city": {
          "types": ["STRING(7)"]
        },
        "country": {
          "types": ["STRING(7)"]
        },
        "state": {
          "types": ["STRING(11)"]
        },
        "street": {
          "types": ["STRING(9)"]
        },
        "zip": {
          "types": ["STRING(5)"]
        }
      }
    ]
  },
  "age": {
    "types": ["NUMBER", "STRING(2)"]
  },
  "name": {
    "types": ["STRING(8, 15)"]
  },
  "personal_data": {
    "types": [
      {
        "gender": {
          "types": ["STRING(4, 6)"]
        },
        "marital_status": {
          "types": ["STRING(6, 7)"]
        }
      },
      {
        "gender": {
          "types": ["STRING(6)"]
        }
      }
    ]
  },
  "phones": {
    "types": [
      "NULL",
      {
        "ARRAY": [
          {
            "mobile": {
              "types": ["STRING(10, 11)"]
            }
          },
          {
            "fax": {
              "types": ["STRING(13)"]
            },
            "mobile": {
              "types": ["STRING(13)"]
            }
          },
          "NUMBER",
          "STRING(10, 17)"
        ]
      }
    ]
  },
  "title": {
    "types": ["STRING(0, 3)"]
  }
}

```
