# schermz

A CLI tool to create a schema from a JSON file.

## Installation

This tool is written in Rust, so you'll need to install the [Rust toolchain](https://www.rust-lang.org/tools/install) to build it.

```bash
cargo install schermz
```

## Usage

```bash
schermz -f <path to json file>
```

## Example

`sample.json`

```json
[
  {
    "name": "John Doe",
    "title": "",
    "age": 43,
    "address": {
      "street": "10 Downing Street",
      "city": "London",
      "zip": "12345"
    },
    "phones": [
      "+44 1234567",
      "+44 2345678",
      123456,
      { "mobile": "+44 3456789" }
    ]
  },
  {
    "name": "Jerry-Pascal Doe",
    "title": "",
    "age": 56,
    "address": {
      "street": "Gr. Weg 3",
      "city": "Potsdam",
      "zip": ""
    },
    "phones": [
      "+49 1234567",
      "+49 2345678",
      "+49 11111111111",
      "+49 301234566",
      123456,
      { "mobile": "+49 3456789" }
    ]
  },
  {
    "name": "Jane Doe",
    "title": "Dr.",
    "age": "66",
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
          "types": [
            "STRING(6, 7)"
          ]
        },
        "street": {
          "types": [
            "STRING(9, 17)"
          ]
        },
        "zip": {
          "types": [
            "STRING(0, 5)"
          ]
        }
      }
    ]
  },
  "age": {
    "types": [
      "NUMBER",
      "STRING(2)"
    ]
  },
  "name": {
    "types": [
      "STRING(8, 16)"
    ]
  },
  "phones": {
    "types": [
      "NULL",
      {
        "ARRAY": [
          {
            "mobile": {
              "types": [
                "STRING(11)"
              ]
            }
          },
          "NUMBER",
          "STRING(11, 15)"
        ]
      }
    ]
  },
  "title": {
    "types": [
      "STRING(0, 3)"
    ]
  }
}
```
