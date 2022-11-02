use std::{fmt::Error, vec};

use kp_core::{H256, U256};
use rlp::{DecoderError, Rlp, RlpStream};

pub struct Account {
    nonce: U256,
    balance: U256,
    root: H256,
    code_hash: H256,
}

impl Account {

    pub fn default() -> Account {
        Account { nonce: U256::default(), balance: U256::default(), root: H256::default(), code_hash: H256::default() }
    }
  /// Create a new account from RLP.
    pub fn from_rlp(rlp: &[u8]) -> Result<Account, rlp::DecoderError> {
        ::rlp::decode::<Account>(rlp)
            .map(|ba| ba.into())
            .map_err(|e| e.into())
    }

    pub fn to_rlp(&self) -> Vec<u8> {
        ::rlp::encode(self).to_vec()
    }

    pub fn nonce(&self)-> &U256 {
        &self.nonce
    }

    pub fn balance(&self) -> &U256 {
        &self.balance
    } 

    pub fn code_hash(&self) -> H256 {
        self.code_hash.clone()
    }

    pub fn inc_nonce(mut self) {
        self.nonce = self.nonce +1;
    }

    pub fn set_code(mut self, code: H256) {
        self.code_hash = code;
    }

    pub fn sub_balance(mut self, v: &U256) {
        assert!(self.balance >= *v);
        self.balance = self.balance - *v;
    }

    pub fn add_balance(mut self, v: &U256) {
        self.balance = self.balance.saturating_add(*v);
    }
}

impl rlp::Encodable for Account {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append(&self.nonce);
        s.append(&self.balance);
        s.append(&self.root);
        s.append(&self.code_hash);
    }
}

impl rlp::Decodable for Account {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(Account {
            nonce: rlp.val_at(0)?,
            balance: rlp.val_at(1)?,
            root: rlp.val_at(2)?,
            code_hash: rlp.val_at(3)?,
        })
    }
}
