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
  -f, --file <FILE>    Path to the JSON file
  -m, --merge-objects  Whether to merge object types into one
  -h, --help           Print help
  -V, --version        Print version
```

## The `-m` argument

When this argument is passed to schermz, all objects for the same key will be merged into one, meaning, if a key can have multiple different object shapes, they will not be listed separately. This is useful when you want to get a general idea of the data, or you trust that the data is consistent.

Here's a simple example:

`sample.json`

```json
[
  {
    "x": {
      "name": "Martin",
      "age": 30
    }
  },
  {
    "x": {
      "name": "Paul"
    }
  }
]
```

### Without `-m` (default)

```bash
schermz -f ./sample.json

{
  "x": {
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
  "x": {
    "types": [
      {
        "age": {
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
