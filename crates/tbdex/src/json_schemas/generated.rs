#[warn(unused_imports)]
#[allow(dead_code)]
pub const GIT_COMMIT_HASH: &str = "7d2fdd03c9405b920b056ab7c7c776a858dc3591";
pub const DEFINITIONS_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/definitions.json",
  "type": "object",
  "definitions": {
    "did": {
      "type": "string",
      "pattern": "^did:([a-z0-9]+):((?:(?:[a-zA-Z0-9._-]|(?:%[0-9a-fA-F]{2}))*:)*((?:[a-zA-Z0-9._-]|(?:%[0-9a-fA-F]{2}))+))((;[a-zA-Z0-9_.:%-]+=[a-zA-Z0-9_.:%-]*)*)(\/[^\#?]*)?([?][^\#]*)?(\#.*)?$"
    },
    "decimalString": {
      "type": "string",
      "pattern": "^([0-9]+(?:[.][0-9]+)?)$"
    }
  }
}"#;
pub const RESOURCE_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/resource.schema.json",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "metadata": {
      "type": "object",
      "properties": {
        "from": {
          "$ref": "definitions.json\#/definitions/did",
          "description": "The PFI's DID"
        },
        "kind": {
          "type": "string",
          "enum": ["offering", "balance"],
          "description": "The resource kind (e.g. Offering)"
        },
        "id": {
          "type": "string",
          "description": "The resource id"
        },
        "createdAt": {
          "type": "string",
          "description": "When the resource was created at. Expressed as ISO8601"
        },
        "updatedAt": {
          "type": "string",
          "description": "When the resource was last updated. Expressed as ISO8601"
        },
        "protocol": {
          "type": "string",
          "description": "Version of the protocol in use (x.x format)"
        }
      },
      "required": ["from", "kind", "id", "createdAt", "protocol"],
      "description": "The metadata object contains fields about the resource and is present for every tbdex resources of all types."
    },
    "data": {
      "description": "The actual resource content",
      "type": "object"
    },
    "signature": {
      "type": "string",
      "description": "Signature that verifies that authenticity and integrity of a message"
    }
  },
  "required": ["metadata", "data", "signature"],
  "description": "ResourceModel"
}
"#;
pub const BALANCE_DATA_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/balance.schema.json",
  "type": "object",
  "properties": {
    "additionalProperties": false,
    "currencyCode": {
      "type": "string",
      "description": "ISO 4217 currency code string"
    },
    "available": {
      "$ref": "definitions.json\#/definitions/decimalString",
      "description": "The amount available to be transacted with"
    }
  },
  "required": [
    "currencyCode",
    "available"
  ]
}
"#;
pub const OFFERING_DATA_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/offering.schema.json",
  "type": "object",
  "properties": {
    "additionalProperties": false,
    "description": {
      "type": "string",
      "description": "Brief description of what is being offered."
    },
    "payin": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "currencyCode": {
          "type": "string",
          "description": "ISO 4217 currency code string"
        },
        "min": {
          "$ref": "definitions.json\#/definitions/decimalString",
          "description": "Minimum amount of currency that can be requested"
        },
        "max": {
          "$ref": "definitions.json\#/definitions/decimalString",
          "description": "Maximum amount of currency that can be requested"
        },
        "methods": {
          "type": "array",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
              "kind": {
                "type": "string",
                "description": "The type of payment method. e.g. BITCOIN_ADDRESS, DEBIT_CARD, etc."
              },
              "name": {
                "type": "string",
                "description": "Payment Method name. Expected to be rendered on screen."
              },
              "description": {
                "type": "string",
                "description": "Blurb containing helpful information about the payment method. Expected to be rendered on screen. e.g. \"segwit addresses only\""
              },
              "group": {
                "type": "string",
                "description": "Value that can be used to group specific payment methods together (e.g. Mobile Money vs. Direct Bank Deposit)."
              },
              "requiredPaymentDetails": {
                "$ref": "http://json-schema.org/draft-07/schema\#",
                "description": "A JSON Schema containing the fields that need to be collected in order to use this payment method"
              },
              "min": {
                "$ref": "definitions.json\#/definitions/decimalString",
                "description": "Minimum amount required to use this payment method."
              },
              "max": {
                "$ref": "definitions.json\#/definitions/decimalString",
                "description": "Maximum amount allowed when using this payment method."
              },
              "fee": {
                "$ref": "definitions.json\#/definitions/decimalString",
                "description": "Fee charged to use this payment method. Absence of this field implies that there is no _additional_ fee associated to the respective payment method."
              }
            },
            "required": ["kind"]
          }
        }
      },
      "required": ["currencyCode", "methods"]
    },
    "payout": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "currencyCode": {
          "type": "string",
          "description": "ISO 4217 currency code string"
        },
        "min": {
          "$ref": "definitions.json\#/definitions/decimalString",
          "description": "Minimum amount of currency that can be requested"
        },
        "max": {
          "$ref": "definitions.json\#/definitions/decimalString",
          "description": "Maximum amount of currency that can be requested"
        },
        "methods": {
          "type": "array",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "properties": {
              "kind": {
                "type": "string",
                "description": "The type of payment method. e.g. BITCOIN_ADDRESS, DEBIT_CARD, etc."
              },
              "name": {
                "type": "string",
                "description": "Payment Method name. Expected to be rendered on screen."
              },
              "description": {
                "type": "string",
                "description": "Blurb containing helpful information about the payment method. Expected to be rendered on screen. e.g. \"segwit addresses only\""
              },
              "group": {
                "type": "string",
                "description": "Value that can be used to group specific payment methods together (e.g. Mobile Money vs. Direct Bank Deposit)."
              },
              "requiredPaymentDetails": {
                "$ref": "http://json-schema.org/draft-07/schema\#",
                "description": "A JSON Schema containing the fields that need to be collected in order to use this payment method"
              },
              "min": {
                "$ref": "definitions.json\#/definitions/decimalString",
                "description": "Minimum amount required to use this payment method."
              },
              "max": {
                "$ref": "definitions.json\#/definitions/decimalString",
                "description": "Maximum amount allowed when using this payment method."
              },
              "fee": {
                "$ref": "definitions.json\#/definitions/decimalString",
                "description": "Fee charged to use this payment method. absence of this field implies that there is no _additional_ fee associated to the respective payment method"
              },
              "estimatedSettlementTime": {
                "type": "number",
                "description": "Estimated time in seconds for the payout to be settled. e.g. 3600 for 1 hour. 0 for instant settlement.",
                "minimum": 0
              }
            },
            "required": ["kind", "estimatedSettlementTime"]
          }
        }
      },
      "required": ["currencyCode", "methods"]
    },
    "payoutUnitsPerPayinUnit": {
      "type": "string",
      "description": "Number of payout currency units for one payin currency unit (i.e 290000 USD for 1 BTC)"
    },
    "requiredClaims": {
      "type": "object",
      "description": "PresentationDefinition that describes the credential(s) the PFI requires in order to provide a quote."
    },
    "cancellation": {
      "type": "object",
      "properties": {
        "enabled": {
          "type": "boolean",
          "description": "Whether cancellation is enabled for this offering"
        },
        "termsUrl": {
          "type": "string",
          "description": "A link to a page that describes the terms of cancellation"
        },
        "terms": {
          "type": "string",
          "description": "A human-readable description of the terms of cancellation in plaintext"
        }
      },
      "required": ["enabled"]
    }
  },
  "required": [
    "description",
    "payin",
    "payout",
    "payoutUnitsPerPayinUnit",
    "cancellation"
  ]
}
"#;
pub const MESSAGE_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/message.schema.json",
  "definitions": {
    "MessageMetadata": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "from": {
          "$ref": "definitions.json\#/definitions/did",
          "description": "The sender's DID"
        },
        "to": {
          "$ref": "definitions.json\#/definitions/did",
          "description": "The recipient's DID"
        },
        "kind": {
          "type": "string",
          "enum": ["rfq", "quote", "order", "orderstatus", "close", "cancel", "orderinstructions"],
          "description": "The message kind (e.g. rfq, quote)"
        },
        "id": {
          "type": "string",
          "description": "The message ID"
        },
        "exchangeId": {
          "type": "string",
          "description": "ID for a 'thread' of messages between Alice <-> PFI. Set by the first message in a thread"
        },
        "externalId": {
          "type": "string",
          "description": "Arbitrary ID for the caller to associate with the message."
        },
        "createdAt": {
          "type": "string",
          "description": "ISO8601 formatted string representing the timestamp"
        },
        "protocol": {
          "type": "string",
          "description": "Version of the protocol in use (x.x format)"
        }
      },
      "required": ["from", "to", "kind", "id", "exchangeId", "createdAt", "protocol"]
    }
  },
  "type": "object",
  "properties": {
    "metadata": {
      "$ref": "\#/definitions/MessageMetadata"
    },
    "data": {
      "type": "object",
      "description": "The actual message content"
    },
    "signature": {
      "type": "string",
      "description": "Signature that verifies the authenticity and integrity of a message"
    },
    "privateData": {
      "type": "object",
      "description": "Private data which can be detached from the payload without disrupting integrity. Only used in RFQs"
    }
  },
  "additionalProperties": false,
  "required": ["metadata", "data", "signature"]
}
"#;
pub const RFQ_DATA_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/rfq.schema.json",
  "type": "object",
  "properties": {
    "additionalProperties": false,
    "offeringId": {
      "type": "string",
      "description": "Offering which Alice would like to get a quote for"
    },
    "claimsHash": {
      "type": "string",
      "description": "Digests of Presentation Submissions that fulfills the requirements included in the respective Offering"
    },
    "payin": {
      "type": "object",
      "properties": {
        "amount": {
          "$ref": "definitions.json\#/definitions/decimalString"
        },
        "kind": {
          "type": "string",
          "description": "Type of payment method e.g. BTC_ADDRESS, DEBIT_CARD, MOMO_MPESA"
        },
        "paymentDetailsHash": {
          "type": "string",
          "description": "Digest of an object containing the properties defined in the respective Offering's requiredPaymentDetails json schema"
        }
      },
      "required": ["amount", "kind"]
    },
    "payout": {
      "type": "object",
      "properties": {
        "kind": {
          "type": "string",
          "description": "Selected payout method from the respective offering"
        },
        "paymentDetailsHash": {
          "type": "string",
          "description": "Digest of an object containing the properties defined in the respective Offering's requiredPaymentDetails json schema"
        }
      },
      "required": ["kind"]
    }
  },
  "required": ["offeringId", "payin", "payout"]
}
"#;
pub const RFQ_PRIVATE_DATA_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/rfq-private.schema.json",
  "type": "object",
  "properties": {
    "additionalProperties": false,
    "salt": {
      "type": "string",
      "description": "Randomly generated cryptographic salt used to hash privateData fields"
    },
    "claims": {
      "type": "array",
      "minItems": 1,
      "items": {
        "type": "string"
      },
      "description": "Presentation Submission that fulfills the requirements included in the respective Offering"
    },
    "payin": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "paymentDetails": {
          "type": "object",
          "description": "An object containing the properties defined in the respective Offering's requiredPaymentDetails json schema"
        }
      }
    },
    "payout": {
      "additionalProperties": false,
      "type": "object",
      "properties": {
        "paymentDetails": {
          "type": "object",
          "description": "An object containing the properties defined in the respective Offering's requiredPaymentDetails json schema"
        }
      }
    }
  },
  "required": ["salt"]
}
"#;
pub const QUOTE_DATA_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/quote.schema.json",
  "definitions": {
    "QuoteDetails": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "currencyCode": {
          "type": "string",
          "description": "ISO 4217 currency code string"
        },
        "subtotal": {
          "$ref": "definitions.json\#/definitions/decimalString",
          "description": "The amount of currency paid for the exchange, excluding fees"
        },
        "fee": {
          "$ref": "definitions.json\#/definitions/decimalString",
          "description": "The amount of currency paid in fees"
        },
        "total": {
          "$ref": "definitions.json\#/definitions/decimalString",
          "description": "The total amount of currency to be paid in or paid out. It is always a sum of subtotal and fee"
        }
      },
      "required": ["currencyCode", "subtotal", "total"]
    }
  },
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "expiresAt": {
      "type": "string",
      "description": "When this quote expires. Expressed as ISO8601"
    },
    "payoutUnitsPerPayinUnit": {
      "type": "string",
      "description": "The exchange rate to convert from payin currency to payout currency. Expressed as an unrounded decimal string."
    },
    "payin": {
      "$ref": "\#/definitions/QuoteDetails"
    },
    "payout": {
      "$ref": "\#/definitions/QuoteDetails"
    }
  },
  "required": ["expiresAt", "payoutUnitsPerPayinUnit", "payin", "payout"]
}
"#;
pub const ORDER_DATA_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/order.schema.json",
  "type": "object",
  "additionalProperties": false,
  "properties": {}
}"#;
pub const ORDER_INSTRUCTIONS_DATA_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/orderinstructions.schema.json",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "payin": {
      "$ref": "\#/definitions/PaymentInstruction"
    },
    "payout": {
      "$ref": "\#/definitions/PaymentInstruction"
    }
  },
  "definitions": {
    "PaymentInstruction": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "link": {
          "type": "string",
          "description": "Link to allow Alice to pay PFI, or be paid by the PFI"
        },
        "instruction": {
          "type": "string",
          "description": "Instruction on how Alice can pay PFI, or how Alice can be paid by the PFI"
        }
      }
    }
  },
  "required": ["payin", "payout"]
}
"#;
pub const CANCEL_DATA_JSON_SCHEMA: &str = r#"{
    "$schema": "http://json-schema.org/draft-07/schema\#",
    "$id": "https://tbdex.dev/cancel.schema.json",
    "type": "object",
    "additionalProperties": false,
    "properties": {
      "reason": {
        "type": "string"
      }
    }
  }"#;
pub const ORDER_STATUS_DATA_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/orderstatus.schema.json",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "status": {
      "type":"string",
      "enum": [
        "PAYIN_PENDING", 
        "PAYIN_INITIATED", 
        "PAYIN_SETTLED", 
        "PAYIN_FAILED", 
        "PAYIN_EXPIRED",
        "PAYOUT_PENDING", 
        "PAYOUT_INITIATED", 
        "PAYOUT_SETTLED", 
        "PAYOUT_FAILED", 
        "REFUND_PENDING", 
        "REFUND_INITIATED", 
        "REFUND_SETTLED", 
        "REFUND_FAILED"
      ]
    },
    "details": {
      "type":"string"
    }
  },
  "required": ["status"]
}"#;
pub const CLOSE_DATA_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema\#",
  "$id": "https://tbdex.dev/close.schema.json",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "reason": {
      "type": "string"
    },
    "success": {
      "type": "boolean"
    }
  }
}"#;
pub const DRAFT_07_JSON_SCHEMA: &str = r#"{
    "$schema": "http://json-schema.org/draft-07/schema\#",
    "$id": "http://json-schema.org/draft-07/schema\#",
    "title": "Core schema meta-schema",
    "definitions": {
        "schemaArray": {
            "type": "array",
            "minItems": 1,
            "items": { "$ref": "\#" }
        },
        "nonNegativeInteger": {
            "type": "integer",
            "minimum": 0
        },
        "nonNegativeIntegerDefault0": {
            "allOf": [
                { "$ref": "\#/definitions/nonNegativeInteger" },
                { "default": 0 }
            ]
        },
        "simpleTypes": {
            "enum": [
                "array",
                "boolean",
                "integer",
                "null",
                "number",
                "object",
                "string"
            ]
        },
        "stringArray": {
            "type": "array",
            "items": { "type": "string" },
            "uniqueItems": true,
            "default": []
        }
    },
    "type": ["object", "boolean"],
    "properties": {
        "$id": {
            "type": "string",
            "format": "uri-reference"
        },
        "$schema": {
            "type": "string",
            "format": "uri"
        },
        "$ref": {
            "type": "string",
            "format": "uri-reference"
        },
        "$comment": {
            "type": "string"
        },
        "title": {
            "type": "string"
        },
        "description": {
            "type": "string"
        },
        "default": true,
        "readOnly": {
            "type": "boolean",
            "default": false
        },
        "writeOnly": {
            "type": "boolean",
            "default": false
        },
        "examples": {
            "type": "array",
            "items": true
        },
        "multipleOf": {
            "type": "number",
            "exclusiveMinimum": 0
        },
        "maximum": {
            "type": "number"
        },
        "exclusiveMaximum": {
            "type": "number"
        },
        "minimum": {
            "type": "number"
        },
        "exclusiveMinimum": {
            "type": "number"
        },
        "maxLength": { "$ref": "\#/definitions/nonNegativeInteger" },
        "minLength": { "$ref": "\#/definitions/nonNegativeIntegerDefault0" },
        "pattern": {
            "type": "string",
            "format": "regex"
        },
        "additionalItems": { "$ref": "\#" },
        "items": {
            "anyOf": [
                { "$ref": "\#" },
                { "$ref": "\#/definitions/schemaArray" }
            ],
            "default": true
        },
        "maxItems": { "$ref": "\#/definitions/nonNegativeInteger" },
        "minItems": { "$ref": "\#/definitions/nonNegativeIntegerDefault0" },
        "uniqueItems": {
            "type": "boolean",
            "default": false
        },
        "contains": { "$ref": "\#" },
        "maxProperties": { "$ref": "\#/definitions/nonNegativeInteger" },
        "minProperties": { "$ref": "\#/definitions/nonNegativeIntegerDefault0" },
        "required": { "$ref": "\#/definitions/stringArray" },
        "additionalProperties": { "$ref": "\#" },
        "definitions": {
            "type": "object",
            "additionalProperties": { "$ref": "\#" },
            "default": {}
        },
        "properties": {
            "type": "object",
            "additionalProperties": { "$ref": "\#" },
            "default": {}
        },
        "patternProperties": {
            "type": "object",
            "additionalProperties": { "$ref": "\#" },
            "propertyNames": { "format": "regex" },
            "default": {}
        },
        "dependencies": {
            "type": "object",
            "additionalProperties": {
                "anyOf": [
                    { "$ref": "\#" },
                    { "$ref": "\#/definitions/stringArray" }
                ]
            }
        },
        "propertyNames": { "$ref": "\#" },
        "const": true,
        "enum": {
            "type": "array",
            "items": true,
            "minItems": 1,
            "uniqueItems": true
        },
        "type": {
            "anyOf": [
                { "$ref": "\#/definitions/simpleTypes" },
                {
                    "type": "array",
                    "items": { "$ref": "\#/definitions/simpleTypes" },
                    "minItems": 1,
                    "uniqueItems": true
                }
            ]
        },
        "format": { "type": "string" },
        "contentMediaType": { "type": "string" },
        "contentEncoding": { "type": "string" },
        "if": { "$ref": "\#" },
        "then": { "$ref": "\#" },
        "else": { "$ref": "\#" },
        "allOf": { "$ref": "\#/definitions/schemaArray" },
        "anyOf": { "$ref": "\#/definitions/schemaArray" },
        "oneOf": { "$ref": "\#/definitions/schemaArray" },
        "not": { "$ref": "\#" }
    },
    "default": true
}
"#;
