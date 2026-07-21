use {codama::Codama, typhoon_idl_generator::plugin::TyphoonPlugin};

#[test]
fn idl_test() {
    let codama = Codama::load("./tests/crate")
        .unwrap()
        .add_plugin(TyphoonPlugin);
    let idl = codama.get_json_idl().unwrap();
    let minify_json = |input: &str| -> String {
        let mut out = String::with_capacity(input.len());
        let mut in_string = false;
        let mut escape = false;

        for c in input.chars() {
            if in_string {
                out.push(c);
                if escape {
                    escape = false;
                } else if c == '\\' {
                    escape = true;
                } else if c == '"' {
                    in_string = false;
                }
            } else if c == '"' {
                in_string = true;
                out.push(c);
            } else if !c.is_whitespace() {
                out.push(c);
            }
        }

        out
    };
    assert_eq!(
        idl,
        minify_json(
            r#"{
  "kind": "rootNode",
  "standard": "codama",
  "version": "1.6.0",
  "program": {
    "kind": "programNode",
    "name": "test",
    "publicKey": "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS",
    "version": "1.0.0",
    "accounts": [
      {
        "kind": "accountNode",
        "name": "counter",
        "data": {
          "kind": "structTypeNode",
          "fields": [
            {
              "kind": "structFieldTypeNode",
              "name": "count",
              "type": {
                "kind": "numberTypeNode",
                "format": "u64",
                "endian": "le"
              }
            },
            {
              "kind": "structFieldTypeNode",
              "name": "another",
              "type": {
                "kind": "definedTypeLinkNode",
                "name": "anotherStruct"
              }
            }
          ]
        },
        "discriminators": [
          {
            "kind": "constantDiscriminatorNode",
            "offset": 0,
            "constant": {
              "kind": "constantValueNode",
              "type": {
                "kind": "bytesTypeNode"
              },
              "value": {
                "kind": "bytesValueNode",
                "data": "p4z99AEAAAA=",
                "encoding": "base64"
              }
            }
          }
        ]
      }
    ],
    "instructions": [
      {
        "kind": "instructionNode",
        "name": "initialize",
        "optionalAccountStrategy": "programId",
        "accounts": [
          {
            "kind": "instructionAccountNode",
            "name": "payer",
            "isWritable": true,
            "isSigner": true,
            "isOptional": false
          },
          {
            "kind": "instructionAccountNode",
            "name": "counter",
            "isWritable": true,
            "isSigner": true,
            "isOptional": false
          },
          {
            "kind": "instructionAccountNode",
            "name": "system",
            "isWritable": false,
            "isSigner": false,
            "isOptional": false
          }
        ],
        "arguments": [
          
        ],
        "discriminators": [
          {
            "kind": "constantDiscriminatorNode",
            "offset": 0,
            "constant": {
              "kind": "constantValueNode",
              "type": {
                "kind": "numberTypeNode",
                "format": "u8",
                "endian": "le"
              },
              "value": {
                "kind": "numberValueNode",
                "number": 0
              }
            }
          }
        ]
      },
      {
        "kind": "instructionNode",
        "name": "increment",
        "optionalAccountStrategy": "programId",
        "accounts": [
          {
            "kind": "instructionAccountNode",
            "name": "counter",
            "isWritable": true,
            "isSigner": false,
            "isOptional": false
          }
        ],
        "arguments": [
          
        ],
        "discriminators": [
          {
            "kind": "constantDiscriminatorNode",
            "offset": 0,
            "constant": {
              "kind": "constantValueNode",
              "type": {
                "kind": "numberTypeNode",
                "format": "u8",
                "endian": "le"
              },
              "value": {
                "kind": "numberValueNode",
                "number": 1
              }
            }
          }
        ]
      },
      {
        "kind": "instructionNode",
        "name": "close",
        "optionalAccountStrategy": "programId",
        "accounts": [
          {
            "kind": "instructionAccountNode",
            "name": "counter",
            "isWritable": true,
            "isSigner": false,
            "isOptional": false
          },
          {
            "kind": "instructionAccountNode",
            "name": "destination",
            "isWritable": true,
            "isSigner": false,
            "isOptional": false
          }
        ],
        "arguments": [
          
        ],
        "discriminators": [
          {
            "kind": "constantDiscriminatorNode",
            "offset": 0,
            "constant": {
              "kind": "constantValueNode",
              "type": {
                "kind": "numberTypeNode",
                "format": "u8",
                "endian": "le"
              },
              "value": {
                "kind": "numberValueNode",
                "number": 2
              }
            }
          }
        ]
      },
      {
        "kind": "instructionNode",
        "name": "randomInstruction",
        "optionalAccountStrategy": "programId",
        "accounts": [
          {
            "kind": "instructionAccountNode",
            "name": "account",
            "isWritable": false,
            "isSigner": false,
            "isOptional": false
          }
        ],
        "arguments": [
          {
            "kind": "instructionArgumentNode",
            "name": "amount",
            "type": {
              "kind": "definedTypeLinkNode",
              "name": "anotherStruct"
            }
          },
          {
            "kind": "instructionArgumentNode",
            "name": "randomContextArgs",
            "type": {
              "kind": "structTypeNode",
              "fields": [
                {
                  "kind": "structFieldTypeNode",
                  "name": "value",
                  "type": {
                    "kind": "numberTypeNode",
                    "format": "u64",
                    "endian": "le"
                  }
                }
              ]
            }
          }
        ],
        "discriminators": [
          {
            "kind": "constantDiscriminatorNode",
            "offset": 0,
            "constant": {
              "kind": "constantValueNode",
              "type": {
                "kind": "numberTypeNode",
                "format": "u8",
                "endian": "le"
              },
              "value": {
                "kind": "numberValueNode",
                "number": 3
              }
            }
          }
        ]
      }
    ],
    "definedTypes": [
      {
        "kind": "definedTypeNode",
        "name": "anotherStruct",
        "type": {
          "kind": "structTypeNode",
          "fields": [
            {
              "kind": "structFieldTypeNode",
              "name": "amount",
              "type": {
                "kind": "numberTypeNode",
                "format": "u64",
                "endian": "le"
              }
            }
          ]
        }
      }
    ],
    "pdas": [

    ],
    "events": [

    ],
    "errors": [
      {
        "kind": "errorNode",
        "name": "error1",
        "code": 0,
        "message": "my custom error"
      }
    ],
    "constants": [

    ]
  },
  "additionalPrograms": [
    
  ]
}"#
        )
    )
}
