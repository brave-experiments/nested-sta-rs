//! The `nested-sta-rs` crate defines the public API for the Nested STAR
//! aggregation mechanism: a modification of the original
//! [STAR](https://github.com/brave-experiments/sta-rs) protocol to
//! allow clients to submit ordered, granular data at the highest
//! resolution that is possible, whilst maintaining k-anonymity.
//! Therefore, Nested STAR provides both higher utility for data
//! aggregation (revealing partial measurements where possible), and
//! better privacy for fine-grained and unique client datapoints.
//!
//! ## Background
//!
//! Specifically, Nested STAR 'nests' or 'layers' an ordered vector of
//! measurements into associated STAR messages, such that each message
//! can only be accessed if the STAR message at the previous layer was
//! included in a successful recovery. The privacy of unrevealed layers
//! is provided using symmetric encryption, that can only be decrypted
//! using a key enclosed in the previous STAR message.
//!
//! ## Example API usage
//!
//! ### Client
//!
//! The Client produces an aggregation message using the Nested STAR
//! message format.
//!
//! ```
//! # use nested_sta_rs::api::*;
//! # use nested_sta_rs::consts::*;
//! # use nested_sta_rs::errors::*;
//! # use nested_sta_rs::format::*;
//! #
//! # struct ExampleClient {}
//! # impl Client for ExampleClient {
//! #   // DO NOT USE THIS FUNCTION IN REAL LIFE
//! #   fn sample_randomness(rsf: &RandomnessSampling, _rs_url: &str) -> Result<MessageGeneration, NestedSTARError> {
//! #     let mut rand_bytes: Vec<[u8; RANDOMNESS_LEN]> = Vec::with_capacity(rsf.input_len());
//! #     for i in 0..rsf.input_len() {
//! #         // NEVER DO THIS
//! #         let input = rsf.input()[i].clone();
//! #         let mut rnd_buf = [0u8; RANDOMNESS_LEN];
//! #         rnd_buf.copy_from_slice(&input[input.len()-32..]);
//! #         rand_bytes.push(rnd_buf.clone());
//! #     }
//! #     MessageGeneration::new(&rsf, rand_bytes)
//! #   }
//! # }
//! let threshold = 10;
//! let epoch = "a";
//! let example_aux = vec![1u8; 3];
//!
//! let measurements = vec!["hello".as_bytes().to_vec(), "world".as_bytes().to_vec()];
//! let rsf = ExampleClient::format_measurement(&measurements, epoch).unwrap();
//! let mgf = ExampleClient::sample_randomness(&rsf, "https://randomness.server").unwrap();
//! ExampleClient::construct_message(&mgf, &example_aux, threshold).unwrap();
//! ```
//!
//! ### Server
//!
//! Server aggregation takes a number of client messages as input, and
//! outputs those measurements that were received from at least
//! `threshold` clients. It also reveals prefixes of full measurements
//! that were received by greater than `threshold` clients.
//!
//! #### Full recovery
//!
//! After receiving at least `threshold` client messages of the same
//! full measurement, the server can run aggregation and reveal the
//! client measurement.
//!
//! ```
//! # use nested_sta_rs::api::*;
//! # use nested_sta_rs::consts::*;
//! # use nested_sta_rs::errors::*;
//! # use nested_sta_rs::format::*;
//! #
//! # struct ExampleClient {}
//! # impl Client for ExampleClient {
//! #   // DO NOT USE THIS FUNCTION IN REAL LIFE
//! #   fn sample_randomness(rsf: &RandomnessSampling, _rs_url: &str) -> Result<MessageGeneration, NestedSTARError> {
//! #     let mut rand_bytes: Vec<[u8; RANDOMNESS_LEN]> = Vec::with_capacity(rsf.input_len());
//! #     for i in 0..rsf.input_len() {
//! #         // NEVER DO THIS
//! #         let input = rsf.input()[i].clone();
//! #         let mut rnd_buf = [0u8; RANDOMNESS_LEN];
//! #         rnd_buf.copy_from_slice(&input[input.len()-32..]);
//! #         rand_bytes.push(rnd_buf.clone());
//! #     }
//! #     MessageGeneration::new(&rsf, rand_bytes)
//! #   }
//! # }
//! # struct ExampleServer {}
//! # impl Server for ExampleServer {}
//!
//! let threshold = 10;
//! let epoch = "a";
//!
//! // construct at least `threshold` client messages with the same measurement
//! let measurements_1 = vec!["hello".as_bytes().to_vec(), "world".as_bytes().to_vec()];
//! let client_messages_to_reveal: Vec<Vec<u8>> = (0..threshold).into_iter().map(|i| {
//!   let example_aux = vec![i as u8; 3];
//!   let rsf = ExampleClient::format_measurement(&measurements_1, epoch).unwrap();
//!   let mgf = ExampleClient::sample_randomness(&rsf, "https://randomness.server").unwrap();
//!   ExampleClient::construct_message(
//!     &mgf,
//!     &example_aux,
//!     threshold,
//!   ).unwrap().as_bytes().to_vec()
//! }).collect();
//!
//! // construct a low number client messages with a different measurement
//! let measurements_2 = vec!["something".as_bytes().to_vec(), "else".as_bytes().to_vec()];
//! let client_messages_to_hide: Vec<Vec<u8>> = (0..2).into_iter().map(|i| {
//!   let example_aux = vec![i as u8; 3];
//!   let rsf = ExampleClient::format_measurement(&measurements_2, epoch).unwrap();
//!   let mgf = ExampleClient::sample_randomness(&rsf, "https://randomness.server").unwrap();
//!   ExampleClient::construct_message(
//!     &mgf,
//!     &example_aux,
//!     threshold,
//!   ).unwrap().as_bytes().to_vec()
//! }).collect();
//!
//! // aggregation reveals the client measurement that reaches the
//! // threshold, the other measurement stays hidden
//! let agg_res = ExampleServer::aggregate(
//!   &[client_messages_to_reveal, client_messages_to_hide].concat(),
//!   threshold,
//!   epoch,
//!   measurements_1.len()
//! );
//! let output = agg_res.outputs();
//! assert_eq!(output.len(), 2);
//! let revealed_output = output.iter().find(|v| v.value() == vec!["hello", "world"]).unwrap();
//! assert_eq!(revealed_output.value(), vec!["hello", "world"]);
//! assert_eq!(revealed_output.occurrences(), 10);
//! (0..10).into_iter().for_each(|i| {
//!   assert_eq!(revealed_output.auxiliary_data()[i], vec![i as u8; 3]);
//! });
//! ```
//!
//! #### Partial recovery
//!
//! Partial recovery allows revealing prefixes of full measurements that
//! are received by enough clients, even when the full measurements
//! themselves stay hidden.
//!
//! ```
//! # use nested_sta_rs::api::*;
//! # use nested_sta_rs::consts::*;
//! # use nested_sta_rs::errors::*;
//! # use nested_sta_rs::format::*;
//! #
//! # struct ExampleClient {}
//! # impl Client for ExampleClient {
//! #   // DO NOT USE THIS FUNCTION IN REAL LIFE
//! #   fn sample_randomness(rsf: &RandomnessSampling, _rs_url: &str) -> Result<MessageGeneration, NestedSTARError> {
//! #     let mut rand_bytes: Vec<[u8; RANDOMNESS_LEN]> = Vec::with_capacity(rsf.input_len());
//! #     for i in 0..rsf.input_len() {
//! #         // NEVER DO THIS
//! #         let input = rsf.input()[i].clone();
//! #         let mut rnd_buf = [0u8; RANDOMNESS_LEN];
//! #         rnd_buf.copy_from_slice(&input[input.len()-32..]);
//! #         rand_bytes.push(rnd_buf.clone());
//! #     }
//! #     MessageGeneration::new(&rsf, rand_bytes)
//! #   }
//! # }
//! # struct ExampleServer {}
//! # impl Server for ExampleServer {}
//!
//! let threshold = 10;
//! let epoch = "a";
//!
//! // construct a low number client messages with the same measurement
//! let measurements_1 = vec!["hello".as_bytes().to_vec(), "world".as_bytes().to_vec()];
//! let client_messages_1: Vec<Vec<u8>> = (0..5).into_iter().map(|i| {
//!   let example_aux = vec![i as u8; 3];
//!   let rsf = ExampleClient::format_measurement(&measurements_1, epoch).unwrap();
//!   let mgf = ExampleClient::sample_randomness(&rsf, "https://randomness.server").unwrap();
//!   ExampleClient::construct_message(
//!     &mgf,
//!     &example_aux,
//!     threshold
//!   ).unwrap().as_bytes().to_vec()
//! }).collect();
//!
//! // construct a low number of measurements that also share a prefix
//! let measurements_2 = vec!["hello".as_bytes().to_vec(), "goodbye".as_bytes().to_vec()];
//! let client_messages_2: Vec<Vec<u8>> = (0..5).into_iter().map(|i| {
//!   let example_aux = vec![i as u8; 3];
//!   let rsf = ExampleClient::format_measurement(&measurements_2, epoch).unwrap();
//!   let mgf = ExampleClient::sample_randomness(&rsf, "https://randomness.server").unwrap();
//!   ExampleClient::construct_message(
//!     &mgf,
//!     &example_aux,
//!     threshold
//!   ).unwrap().as_bytes().to_vec()
//! }).collect();
//!
//! // aggregation reveals the partial client measurement `vec!["hello"]`,
//! // but the full measurements stay hidden
//! let agg_res = ExampleServer::aggregate(
//!   &[client_messages_1, client_messages_2].concat(),
//!   threshold,
//!   epoch,
//!   measurements_1.len()
//! );
//! let output = agg_res.outputs();
//! assert_eq!(output.len(), 1);
//! assert_eq!(output[0].value(), vec!["hello"]);
//! assert_eq!(output[0].occurrences(), 10);
//! (0..10).into_iter().for_each(|i| {
//!   let val = i % 5;
//!   assert_eq!(output[0].auxiliary_data()[i], vec![val as u8; 3]);
//! });
//! ```
mod internal;

pub mod api;
pub mod format;

pub mod consts {
  pub const MAX_MEASUREMENT_LEN: usize = 32;
  pub const RANDOMNESS_LEN: usize = 32;
}

pub mod errors {
  use std::fmt;

  #[derive(Debug, Clone, PartialEq)]
  pub enum NestedSTARError {
    LongMeasurementError,
    ShareRecoveryFailedError,
    ClientMeasurementMismatchError(String, String),
    LayerEncryptionKeysError(usize, usize),
    NumMeasurementLayersError(usize, usize),
    SerdeJSONError,
  }

  impl std::error::Error for NestedSTARError {}

  impl fmt::Display for NestedSTARError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
        NestedSTARError::LongMeasurementError => write!(f, "Input measurement contains long entry, maximum length is {}", crate::consts::MAX_MEASUREMENT_LEN),
        NestedSTARError::ShareRecoveryFailedError => write!(f, "Internal share recovery failed"),
        NestedSTARError::ClientMeasurementMismatchError(original, received) => write!(f, "Clients sent differing measurement for identical share sets, original: {}, received: {}", original, received),
        NestedSTARError::LayerEncryptionKeysError(nkeys, nlayers) => write!(f, "Number of encryption keys ({}) provided for nested encryptions is not compatible with number of layers specified ({}).", nkeys, nlayers),
        NestedSTARError::NumMeasurementLayersError(current, expected) => write!(f, "Number of inferred measurement layers is {}, but expected is {}.", current, expected),
        NestedSTARError::SerdeJSONError => write!(f, "An error occurred during JSON serialization/deserialization."),
      }
    }
  }
}