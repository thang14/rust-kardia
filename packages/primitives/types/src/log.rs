use alloc::vec::Vec;
use kp_core::{H160, H256};
use rlp_derive::{RlpDecodable, RlpEncodable};

#[derive(Clone, Debug, PartialEq, Eq, RlpEncodable, RlpDecodable)]
#[cfg_attr(
	feature = "with-codec",
	derive(codec::Encode, codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Log {
	pub address: H160,
	pub topics: Vec<H256>,
	pub data: Vec<u8>,
}

impl Log {
	pub fn new(address: H160, topics: Vec<H256>, data: Vec<u8>) -> Log{
		Log { address: address, topics: topics, data: data }
	}
}