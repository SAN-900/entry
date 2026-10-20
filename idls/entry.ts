/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/entry.json`.
 */
export type Entry = {
  "address": "EsW1tbFTY87F7k669NzJ1mdp7kVgTb2DXyANvpSoU2y9",
  "metadata": {
    "name": "entry",
    "version": "0.1.0",
    "spec": "0.1.0",
    "description": "ENTRY fair ticketing program"
  },
  "instructions": [
    {
      "name": "buyResale",
      "discriminator": [
        71,
        230,
        159,
        123,
        90,
        231,
        111,
        104
      ],
      "accounts": [
        {
          "name": "buyer",
          "writable": true,
          "signer": true
        },
        {
          "name": "event",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "event.organizer",
                "account": "event"
              },
              {
                "kind": "account",
                "path": "event.event_id",
                "account": "event"
              }
            ]
          },
          "relations": [
            "listing"
          ]
        },
        {
          "name": "listing",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  105,
                  115,
                  116,
                  105,
                  110,
                  103
                ]
              },
              {
                "kind": "account",
                "path": "event"
              },
              {
                "kind": "account",
                "path": "listing.asset_id",
                "account": "listing"
              },
              {
                "kind": "account",
                "path": "listing.seller",
                "account": "listing"
              }
            ]
          }
        },
        {
          "name": "seller",
          "writable": true
        },
        {
          "name": "checkins",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  104,
                  101,
                  99,
                  107,
                  105,
                  110,
                  115
                ]
              },
              {
                "kind": "account",
                "path": "event"
              }
            ]
          }
        },
        {
          "name": "purchaseCounter",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  117,
                  114,
                  99,
                  104,
                  97,
                  115,
                  101,
                  115
                ]
              },
              {
                "kind": "account",
                "path": "event"
              },
              {
                "kind": "account",
                "path": "buyer"
              }
            ]
          }
        },
        {
          "name": "config",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "paymentMint"
        },
        {
          "name": "buyerUsdc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "buyer"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "sellerUsdc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "seller"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "organizerUsdc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "event.organizer",
                "account": "event"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "feeVaultUsdc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "config.fee_vault",
                "account": "config"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "treeConfig",
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "event.merkle_tree",
                "account": "event"
              }
            ],
            "program": {
              "kind": "account",
              "path": "bubblegumProgram"
            }
          }
        },
        {
          "name": "merkleTree",
          "writable": true
        },
        {
          "name": "collection"
        },
        {
          "name": "leafOwner"
        },
        {
          "name": "eventAuthority"
        },
        {
          "name": "bubblegumProgram",
          "address": "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY"
        },
        {
          "name": "compressionProgram",
          "address": "cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK"
        },
        {
          "name": "logWrapper",
          "address": "noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV"
        },
        {
          "name": "mplCoreProgram",
          "address": "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "buyResaleArgs"
            }
          }
        }
      ]
    },
    {
      "name": "cancelListing",
      "discriminator": [
        41,
        183,
        50,
        232,
        230,
        233,
        157,
        70
      ],
      "accounts": [
        {
          "name": "seller",
          "writable": true,
          "signer": true,
          "relations": [
            "listing"
          ]
        },
        {
          "name": "event",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "event.organizer",
                "account": "event"
              },
              {
                "kind": "account",
                "path": "event.event_id",
                "account": "event"
              }
            ]
          },
          "relations": [
            "listing"
          ]
        },
        {
          "name": "listing",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  105,
                  115,
                  116,
                  105,
                  110,
                  103
                ]
              },
              {
                "kind": "account",
                "path": "event"
              },
              {
                "kind": "account",
                "path": "listing.asset_id",
                "account": "listing"
              },
              {
                "kind": "account",
                "path": "seller"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "checkInBatch",
      "discriminator": [
        79,
        154,
        240,
        82,
        164,
        148,
        224,
        81
      ],
      "accounts": [
        {
          "name": "gate",
          "signer": true
        },
        {
          "name": "event",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "event.organizer",
                "account": "event"
              },
              {
                "kind": "account",
                "path": "event.event_id",
                "account": "event"
              }
            ]
          }
        },
        {
          "name": "checkins",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  104,
                  101,
                  99,
                  107,
                  105,
                  110,
                  115
                ]
              },
              {
                "kind": "account",
                "path": "event"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "nonces",
          "type": {
            "vec": "u64"
          }
        }
      ]
    },
    {
      "name": "createEvent",
      "discriminator": [
        49,
        219,
        29,
        203,
        22,
        98,
        100,
        87
      ],
      "accounts": [
        {
          "name": "organizer",
          "writable": true,
          "signer": true
        },
        {
          "name": "event",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "organizer"
              },
              {
                "kind": "arg",
                "path": "args.event_id"
              }
            ]
          }
        },
        {
          "name": "checkins",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  104,
                  101,
                  99,
                  107,
                  105,
                  110,
                  115
                ]
              },
              {
                "kind": "account",
                "path": "event"
              }
            ]
          }
        },
        {
          "name": "merkleTree"
        },
        {
          "name": "treeConfig",
          "pda": {
            "seeds": [
              {
                "kind": "arg",
                "path": "args.merkle_tree"
              }
            ],
            "program": {
              "kind": "account",
              "path": "bubblegumProgram"
            }
          }
        },
        {
          "name": "collection"
        },
        {
          "name": "bubblegumProgram",
          "address": "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY"
        },
        {
          "name": "mplCoreProgram",
          "address": "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "createEventArgs"
            }
          }
        }
      ]
    },
    {
      "name": "initConfig",
      "discriminator": [
        23,
        235,
        115,
        232,
        168,
        96,
        1,
        231
      ],
      "accounts": [
        {
          "name": "admin",
          "writable": true,
          "signer": true
        },
        {
          "name": "program",
          "address": "EsW1tbFTY87F7k669NzJ1mdp7kVgTb2DXyANvpSoU2y9"
        },
        {
          "name": "programData"
        },
        {
          "name": "feeVault"
        },
        {
          "name": "config",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "feeBps",
          "type": "u16"
        }
      ]
    },
    {
      "name": "listForResale",
      "discriminator": [
        235,
        101,
        201,
        204,
        83,
        163,
        213,
        243
      ],
      "accounts": [
        {
          "name": "seller",
          "writable": true,
          "signer": true
        },
        {
          "name": "event",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "event.organizer",
                "account": "event"
              },
              {
                "kind": "account",
                "path": "event.event_id",
                "account": "event"
              }
            ]
          }
        },
        {
          "name": "checkins",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  104,
                  101,
                  99,
                  107,
                  105,
                  110,
                  115
                ]
              },
              {
                "kind": "account",
                "path": "event"
              }
            ]
          }
        },
        {
          "name": "listing",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  108,
                  105,
                  115,
                  116,
                  105,
                  110,
                  103
                ]
              },
              {
                "kind": "account",
                "path": "event"
              },
              {
                "kind": "account",
                "path": "event.asset_id(nonce)",
                "account": "event"
              },
              {
                "kind": "account",
                "path": "seller"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "nonce",
          "type": "u64"
        },
        {
          "name": "askPrice",
          "type": "u64"
        }
      ]
    },
    {
      "name": "purchase",
      "discriminator": [
        21,
        93,
        113,
        154,
        193,
        160,
        242,
        168
      ],
      "accounts": [
        {
          "name": "buyer",
          "writable": true,
          "signer": true
        },
        {
          "name": "gatekeeper",
          "signer": true
        },
        {
          "name": "event",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "event.organizer",
                "account": "event"
              },
              {
                "kind": "account",
                "path": "event.event_id",
                "account": "event"
              }
            ]
          }
        },
        {
          "name": "purchaseCounter",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  117,
                  114,
                  99,
                  104,
                  97,
                  115,
                  101,
                  115
                ]
              },
              {
                "kind": "account",
                "path": "event"
              },
              {
                "kind": "account",
                "path": "buyer"
              }
            ]
          }
        },
        {
          "name": "config",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  99,
                  111,
                  110,
                  102,
                  105,
                  103
                ]
              }
            ]
          }
        },
        {
          "name": "paymentMint"
        },
        {
          "name": "buyerUsdc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "buyer"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "organizerUsdc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "event.organizer",
                "account": "event"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "feeVaultUsdc",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "config.fee_vault",
                "account": "config"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "paymentMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "treeConfig",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "event.merkle_tree",
                "account": "event"
              }
            ],
            "program": {
              "kind": "account",
              "path": "bubblegumProgram"
            }
          }
        },
        {
          "name": "merkleTree",
          "writable": true
        },
        {
          "name": "collection",
          "writable": true
        },
        {
          "name": "eventAuthority"
        },
        {
          "name": "mplCoreCpiSigner",
          "address": "CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk"
        },
        {
          "name": "bubblegumProgram",
          "address": "BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY"
        },
        {
          "name": "compressionProgram",
          "address": "cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK"
        },
        {
          "name": "logWrapper",
          "address": "noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV"
        },
        {
          "name": "mplCoreProgram",
          "address": "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "args",
          "type": {
            "defined": {
              "name": "purchaseArgs"
            }
          }
        }
      ]
    },
    {
      "name": "setActive",
      "discriminator": [
        29,
        16,
        225,
        132,
        38,
        216,
        206,
        33
      ],
      "accounts": [
        {
          "name": "organizer",
          "signer": true,
          "relations": [
            "event"
          ]
        },
        {
          "name": "event",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "organizer"
              },
              {
                "kind": "account",
                "path": "event.event_id",
                "account": "event"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "active",
          "type": "bool"
        }
      ]
    },
    {
      "name": "setGateAuthority",
      "discriminator": [
        183,
        5,
        42,
        140,
        101,
        46,
        152,
        74
      ],
      "accounts": [
        {
          "name": "organizer",
          "signer": true,
          "relations": [
            "event"
          ]
        },
        {
          "name": "event",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  101,
                  118,
                  101,
                  110,
                  116
                ]
              },
              {
                "kind": "account",
                "path": "organizer"
              },
              {
                "kind": "account",
                "path": "event.event_id",
                "account": "event"
              }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "gateAuthority",
          "type": "pubkey"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "checkIns",
      "discriminator": [
        139,
        150,
        21,
        199,
        139,
        104,
        47,
        151
      ]
    },
    {
      "name": "config",
      "discriminator": [
        155,
        12,
        170,
        224,
        30,
        250,
        204,
        130
      ]
    },
    {
      "name": "event",
      "discriminator": [
        125,
        192,
        125,
        158,
        9,
        115,
        152,
        233
      ]
    },
    {
      "name": "listing",
      "discriminator": [
        218,
        32,
        50,
        73,
        43,
        134,
        26,
        58
      ]
    },
    {
      "name": "purchaseCounter",
      "discriminator": [
        128,
        113,
        14,
        135,
        227,
        138,
        221,
        215
      ]
    }
  ],
  "events": [
    {
      "name": "checkInBatchEvent",
      "discriminator": [
        155,
        39,
        195,
        100,
        0,
        183,
        216,
        59
      ]
    },
    {
      "name": "ticketPurchased",
      "discriminator": [
        108,
        59,
        246,
        95,
        84,
        145,
        13,
        71
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "feeTooHigh",
      "msg": "Fee exceeds the configured maximum"
    },
    {
      "code": 6001,
      "name": "invalidPrice",
      "msg": "Event price must be greater than zero"
    },
    {
      "code": 6002,
      "name": "invalidSupply",
      "msg": "Supply must be greater than zero"
    },
    {
      "code": 6003,
      "name": "invalidResaleCap",
      "msg": "Resale cap must be at least 100%"
    },
    {
      "code": 6004,
      "name": "invalidRoyalty",
      "msg": "Royalty cannot exceed the resale premium"
    },
    {
      "code": 6005,
      "name": "eventNotActive",
      "msg": "Event is not active"
    },
    {
      "code": 6006,
      "name": "soldOut",
      "msg": "Event supply has been exhausted"
    },
    {
      "code": 6007,
      "name": "resaleCapExceeded",
      "msg": "Asking price exceeds the face-price resale cap"
    },
    {
      "code": 6008,
      "name": "alreadyListed",
      "msg": "Ticket is already listed"
    },
    {
      "code": 6009,
      "name": "notListingSeller",
      "msg": "Listing does not belong to the signer"
    },
    {
      "code": 6010,
      "name": "alreadyCheckedIn",
      "msg": "Ticket has already been checked in"
    },
    {
      "code": 6011,
      "name": "unauthorizedGate",
      "msg": "Signer is not an authorized gate operator"
    },
    {
      "code": 6012,
      "name": "unauthorizedConfigInitializer",
      "msg": "Config initialization must be signed by the program upgrade authority"
    },
    {
      "code": 6013,
      "name": "invalidProgramData",
      "msg": "Program data account does not belong to this program"
    },
    {
      "code": 6014,
      "name": "arithmeticOverflow",
      "msg": "Arithmetic overflow"
    },
    {
      "code": 6015,
      "name": "ticketIdOutOfRange",
      "msg": "Ticket identifier is outside the event supply"
    },
    {
      "code": 6016,
      "name": "invalidAssetId",
      "msg": "Compressed asset identifier does not match the tree and leaf nonce"
    },
    {
      "code": 6017,
      "name": "invalidMerkleProofAccounts",
      "msg": "Merkle proof contains too many accounts or writable/signing accounts"
    },
    {
      "code": 6018,
      "name": "unauthorizedOrganizer",
      "msg": "Only the event organizer may register inventory tickets"
    },
    {
      "code": 6019,
      "name": "notTicketOwner",
      "msg": "Signer is not the owner recorded for this ticket"
    },
    {
      "code": 6020,
      "name": "ticketNotPurchased",
      "msg": "Ticket must be purchased before it can be listed for resale"
    },
    {
      "code": 6021,
      "name": "invalidPaymentMint",
      "msg": "Payment mint must be canonical six-decimal USDC on mainnet or devnet"
    },
    {
      "code": 6022,
      "name": "invalidBubblegumProof",
      "msg": "Bubblegum proof fields do not describe one consistent ticket leaf"
    },
    {
      "code": 6023,
      "name": "invalidBubblegumFlags",
      "msg": "Ticket leaf freeze flags do not describe the required frozen/unfrozen transition"
    },
    {
      "code": 6024,
      "name": "selfPurchase",
      "msg": "Wallet cannot buy its own ticket"
    },
    {
      "code": 6025,
      "name": "invalidCheckInAccount",
      "msg": "Invalid event check-in bitmap account"
    },
    {
      "code": 6026,
      "name": "ticketAlreadyCheckedIn",
      "msg": "Ticket has already been checked in"
    },
    {
      "code": 6027,
      "name": "checkInBatchTooLarge",
      "msg": "Check-in batch exceeds the supported maximum"
    },
    {
      "code": 6028,
      "name": "unauthorizedGatekeeper",
      "msg": "Gatekeeper must sign this purchase"
    },
    {
      "code": 6029,
      "name": "invalidEventTree",
      "msg": "Tree must be private, delegated to the event PDA, and empty"
    },
    {
      "code": 6030,
      "name": "eventInactive",
      "msg": "Event is inactive"
    },
    {
      "code": 6031,
      "name": "invalidMetadata",
      "msg": "Metadata field exceeds the supported size"
    }
  ],
  "types": [
    {
      "name": "buyResaleArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "proofs",
            "type": {
              "defined": {
                "name": "ticketTransferProofs"
              }
            }
          }
        ]
      }
    },
    {
      "name": "checkInBatchEvent",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "event",
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "type": "u64"
          },
          {
            "name": "gateAuthority",
            "type": "pubkey"
          },
          {
            "name": "checkedInAt",
            "type": "i64"
          },
          {
            "name": "alreadyCheckedIn",
            "type": "bool"
          }
        ]
      }
    },
    {
      "name": "checkIns",
      "serialization": "bytemuck",
      "repr": {
        "kind": "c"
      },
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "event",
            "type": "pubkey"
          },
          {
            "name": "supply",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "config",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "feeVault",
            "type": "pubkey"
          },
          {
            "name": "feeBps",
            "type": "u16"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "createEventArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "eventId",
            "type": "u64"
          },
          {
            "name": "merkleTree",
            "type": "pubkey"
          },
          {
            "name": "collection",
            "type": "pubkey"
          },
          {
            "name": "paymentMint",
            "type": "pubkey"
          },
          {
            "name": "gateAuthority",
            "type": "pubkey"
          },
          {
            "name": "gatekeeper",
            "type": "pubkey"
          },
          {
            "name": "price",
            "type": "u64"
          },
          {
            "name": "supply",
            "type": "u32"
          },
          {
            "name": "maxPerWallet",
            "type": "u8"
          },
          {
            "name": "resaleCapBps",
            "type": "u16"
          },
          {
            "name": "royaltyBps",
            "type": "u16"
          },
          {
            "name": "ticketName",
            "type": "string"
          },
          {
            "name": "ticketSymbol",
            "type": "string"
          },
          {
            "name": "ticketUri",
            "type": "string"
          }
        ]
      }
    },
    {
      "name": "event",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "organizer",
            "type": "pubkey"
          },
          {
            "name": "eventId",
            "type": "u64"
          },
          {
            "name": "gateAuthority",
            "type": "pubkey"
          },
          {
            "name": "gatekeeper",
            "type": "pubkey"
          },
          {
            "name": "paymentMint",
            "type": "pubkey"
          },
          {
            "name": "merkleTree",
            "type": "pubkey"
          },
          {
            "name": "collection",
            "type": "pubkey"
          },
          {
            "name": "price",
            "type": "u64"
          },
          {
            "name": "supply",
            "type": "u32"
          },
          {
            "name": "sold",
            "type": "u32"
          },
          {
            "name": "maxPerWallet",
            "type": "u8"
          },
          {
            "name": "resaleCapBps",
            "type": "u16"
          },
          {
            "name": "royaltyBps",
            "type": "u16"
          },
          {
            "name": "ticketName",
            "type": "string"
          },
          {
            "name": "ticketSymbol",
            "type": "string"
          },
          {
            "name": "ticketUri",
            "type": "string"
          },
          {
            "name": "active",
            "type": "bool"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "listing",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "event",
            "type": "pubkey"
          },
          {
            "name": "assetId",
            "type": "pubkey"
          },
          {
            "name": "leafNonce",
            "type": "u64"
          },
          {
            "name": "seller",
            "type": "pubkey"
          },
          {
            "name": "askPrice",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "purchaseArgs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "expectedNonce",
            "docs": [
              "Protects the buyer's precomputed append proof from tree races."
            ],
            "type": "u64"
          },
          {
            "name": "freezeRoot",
            "docs": [
              "Root expected immediately after minting this leaf; Bubblegum verifies it during freeze."
            ],
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          }
        ]
      }
    },
    {
      "name": "purchaseCounter",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "event",
            "type": "pubkey"
          },
          {
            "name": "wallet",
            "type": "pubkey"
          },
          {
            "name": "count",
            "type": "u8"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "ticketLeafProof",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "root",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "dataHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "creatorHash",
            "type": {
              "array": [
                "u8",
                32
              ]
            }
          },
          {
            "name": "assetDataHash",
            "type": {
              "option": {
                "array": [
                  "u8",
                  32
                ]
              }
            }
          },
          {
            "name": "flags",
            "type": {
              "option": "u8"
            }
          },
          {
            "name": "nonce",
            "type": "u64"
          },
          {
            "name": "index",
            "type": "u32"
          }
        ]
      }
    },
    {
      "name": "ticketPurchased",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "event",
            "type": "pubkey"
          },
          {
            "name": "buyer",
            "type": "pubkey"
          },
          {
            "name": "nonce",
            "type": "u64"
          },
          {
            "name": "assetId",
            "type": "pubkey"
          },
          {
            "name": "price",
            "type": "u64"
          },
          {
            "name": "fee",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "ticketTransferProofs",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "thaw",
            "docs": [
              "Proof against the currently frozen leaf, consumed by thawV2."
            ],
            "type": {
              "defined": {
                "name": "ticketLeafProof"
              }
            }
          },
          {
            "name": "transfer",
            "docs": [
              "Proof against the thawed leaf, consumed by transferV2."
            ],
            "type": {
              "defined": {
                "name": "ticketLeafProof"
              }
            }
          },
          {
            "name": "refreeze",
            "docs": [
              "Proof against the transferred leaf, consumed by freezeV2."
            ],
            "type": {
              "defined": {
                "name": "ticketLeafProof"
              }
            }
          }
        ]
      }
    }
  ]
};
