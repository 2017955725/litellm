# String enum conversion survey

Surveyed the Rust workspace source and tests in the working tree on 2026-10-05, including uncommitted files. Searched manual Serde implementations, string conversion traits, Strum attributes, Serde conversion attributes, string accessors, and enum declarations. This inventory describes that snapshot and must be checked against current source before migration

## Proposed migration

Eight types across six source files in three crates have replaceable string conversion glue. Seven can use Strum for their string mappings; `CallKey` can replace only its Serde adapter

| Type | Source relative to `litellm-rust` | Existing glue | Proposed change |
| --- | --- | --- | --- |
| `ResponsesWsEventType` | `crates/llms-types/src/formats/responses/streaming_websocket.rs` | Manual Serialize and Deserialize, including a repeated spelling match | Add EnumString, Display, DeserializeFromStr, and SerializeDisplay; retain AsRefStr and the string schema |
| `ContentBlockType` | `crates/llms-types/src/formats/messages/content.rs` | Strum plus Serde from/into and two conversion implementations | Use serde_with derives, preserve the string schema and any required public conversions |
| `MessageRole` | `crates/llms-types/src/formats/messages/metadata.rs` | Strum plus Serde from/into and two conversion implementations | Use serde_with derives, preserve AsRefStr, the string schema, and any required public conversions |
| `MessageType` | `crates/llms-types/src/formats/messages/metadata.rs` | Same pattern as MessageRole | Same change |
| `StopReason` | `crates/llms-types/src/formats/messages/metadata.rs` | Same pattern as MessageRole | Same change |
| `OperationStatus` | `crates/llms/src/azure_ai/ocr/document_intelligence/transformation.rs` | Manual Deserialize and Display repeat the same status spellings | Add EnumString, Display, and DeserializeFromStr; preserve exact spellings and Unknown(String), without adding Serialize |
| `Integration` | `crates/traces/src/normalize/metadata.rs` | Strum plus Serde from/into and two conversion implementations | Use serde_with derives, preserve kebab-case values and unknown-string output |
| `CallKey` | `crates/traces/src/normalize/mod.rs` | Serde try_from, a delegating TryFrom implementation, and manual Serialize | Use DeserializeFromStr and SerializeDisplay while retaining the custom FromStr, Display, validation, and required public conversions |

The traces crate needs `serde_with.workspace = true`; llms-types and llms already depend on it. No dependency upgrade or shared macro change is needed

The existing ResponsesWsEventType test named `event_type_round_trips_known_and_unknown_values` only checks deserialization of a serialized input string. Extend it to serialize the parsed enum and compare against the input, so an incorrect enum serializer actually fails the test

For these candidates, verify exact known outputs, unknown and empty string preservation where accepted, non-string rejection, existing public conversion callers, and schema equivalence. For CallKey also verify prefix/ID validation and errors. Keep the parser's existing rejection behavior

## Exceptions and nearby patterns

| Pattern | Examples | Decision |
| --- | --- | --- |
| Closed enums already using Serde derives | ReasoningEffort, EffortLevel, Speed, KeyManagementSystem, TraceTable, JsonKind | Keep plain Serde derives unless a separate change demonstrates value in sharing their string mapping |
| Different parser acceptance rules | ObservationType has strict lowercase Serde values but case-insensitive Strum parsing | Keep the distinction; routing JSON through FromStr would broaden accepted inputs |
| Types with no Serde contract | AnthropicBeta, OidcProvider, ReadQuery, AzureCredentialType, internal operation/metadata discriminators | Keep existing Strum use; do not add serialization just for uniformity |
| Structured string parsing | BetaSet, OidcReference, CallKey | Keep custom parsing of delimiters and payloads; CallKey's Serde wrapper is the separate candidate above |
| TLS configuration parsing | KeyExchangeGroup, Tls12CipherSuite, CipherToken, CipherSelection | Keep trimming, aliases, typed errors, and selection logic outside this migration |
| Provider selection | InvokeProvider | Keep its existing Unsupported fallback and transformation ownership; it has no Serde contract |
| JSON shape handling | TextValue, Recognized, Nullable, MessageBatch, ConverseContentBlock, Discovery | Keep JSON visitors, untagged/tagged payload handling, and permissive projections |
| Non-string serialization | Pickled, AttributeJson, EncodedRow, JSON budget seeds, numeric serde_compat adapters | Keep their format-specific serialization and validation |

No source migration was performed for this survey. The proposed migration is the eight types above, their affected tests and schema declarations, and the traces dependency declaration
