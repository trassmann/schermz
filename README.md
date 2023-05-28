# schermz

A CLI tool to create a schema from a JSON file.

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
    "age": 43,
    "address": {
      "street": "10 Downing Street",
      "city": "London"
    },
    "phones": ["+44 1234567", "+44 2345678"]
  },
  {
    "name": "Jane Doe",
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
      [
        "STRING"
      ],
      "NULL"
    ]
  }
}
```
