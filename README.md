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
      "city": "London"
    },
    "phones": ["+44 1234567", "+44 2345678", 123456]
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
            "STRING"
          ]
        },
        "street": {
          "types": [
            "STRING"
          ]
        }
      }
    ]
  },
  "age": {
    "types": [
      "NUMBER",
      "STRING"
    ]
  },
  "name": {
    "types": [
      "STRING"
    ]
  },
  "phones": {
    "types": [
      "ARRAY(STRING, NUMBER)",
      "NULL"
    ]
  },
  "title": {
    "types": [
      "EMPTY_STRING",
      "STRING"
    ]
  }
}
```
