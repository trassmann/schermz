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
          "types": [
            "STRING(6)"
          ]
        },
        "country_code": {
          "types": [
            "STRING(2)"
          ]
        },
        "street": {
          "types": [
            "STRING(17)"
          ]
        },
        "zip": {
          "types": [
            "STRING(5)"
          ]
        }
      },
      {
        "city": {
          "types": [
            "STRING(8)"
          ]
        },
        "country": {
          "types": [
            "STRING(3)"
          ]
        },
        "country_code": {
          "types": [
            "STRING(2)"
          ]
        },
        "state": {
          "types": [
            "STRING(10)"
          ]
        },
        "street": {
          "types": [
            "STRING(14)"
          ]
        },
        "zip": {
          "types": [
            "STRING(8)"
          ]
        }
      },
      {
        "city": {
          "types": [
            "STRING(7)"
          ]
        },
        "country": {
          "types": [
            "STRING(7)"
          ]
        },
        "state": {
          "types": [
            "STRING(11)"
          ]
        },
        "street": {
          "types": [
            "STRING(9)"
          ]
        },
        "zip": {
          "types": [
            "STRING(5)"
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
      "STRING(8, 15)"
    ]
  },
  "personal_data": {
    "types": [
      {
        "gender": {
          "types": [
            "STRING(4, 6)"
          ]
        },
        "marital_status": {
          "types": [
            "STRING(6, 7)"
          ]
        }
      },
      {
        "gender": {
          "types": [
            "STRING(6)"
          ]
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
              "types": [
                "STRING(10, 11)"
              ]
            }
          },
          {
            "fax": {
              "types": [
                "STRING(13)"
              ]
            },
            "mobile": {
              "types": [
                "STRING(13)"
              ]
            }
          },
          "NUMBER",
          "STRING(10, 17)"
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
