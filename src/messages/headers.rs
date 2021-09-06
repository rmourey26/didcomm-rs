use rand::{
    Rng,
};
use std::{
    time::SystemTime,
    collections::HashMap,
};
use crate::{
    Error,
    Jwk,
    crypto::{
        CryptoAlgorithm,
        SignatureAlgorithm
    },
};
use super::{MessageType, PriorClaims};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct DidcommHeader {
    pub id: String,
    #[serde(rename = "type")]
    pub m_type: MessageType,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_time: Option<u64>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) other: HashMap<String, String>,
    /// A JWT, with sub: new DID and iss: prior DID,
    /// with a signature from a key authorized by prior DID.
    #[serde(skip_serializing_if = "Option::is_none")]
    from_prior: Option<PriorClaims>,
}

impl DidcommHeader {
    /// Constructor function with ~default values.
    pub fn new() -> Self {
        DidcommHeader {
            id: DidcommHeader::gen_random_id(),
            m_type: MessageType::DidcommRaw,
            to: vec!(String::default()),
            from: Some(String::default()),
            created_time: None,
            expires_time: None,
            from_prior: None,
            other: HashMap::new(),
        }
    }
    /// Generates random `id`
    /// TODO: Should this be public?
    pub fn gen_random_id() -> String {
        let id_number: usize = rand::thread_rng().gen();
        format!("{}", id_number)
    }
    /// Getter method for `from_prior` retreival
    ///
    pub fn from_prior(&self) -> &Option<PriorClaims> {
        &self.from_prior
    }
    /// Creates set of DIDComm related headers with the static forward type
    ///
    pub fn forward(to: Vec<String>, from: Option<String>, expires_time: Option<u64>) -> Result<Self, Error> {
        Ok(DidcommHeader {
            id: DidcommHeader::gen_random_id(),
            m_type: MessageType::DidcommRaw,
            to,
            from,
            created_time: Some(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs()),
            expires_time,
            ..DidcommHeader::new()
        })
    }
}

impl Default for DidcommHeader {
    fn default() -> Self {
        DidcommHeader::new()
    }
}

/// JWM Header as specifiead in [RFC](https://tools.ietf.org/html/draft-looker-jwm-01#section-2.3)
/// With single deviation - allows raw text JWM to support DIDComm spec
///
/// Designed to work for both [JWE](https://tools.ietf.org/html/rfc7516) and [JWS](https://tools.ietf.org/html/rfc7515) message types.
///
/// `iv` property is not explicitly listed in the registered properties on the RFC but is present
///     within example lists - used here as DIDComm crypto nonce sharing property.
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JwmHeader {
    pub typ: String,
    // Some(String) if JWM is JWE encrypted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enc: Option<String>,
    // None if raw text message, Some(key ID) otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skid: Option<String>,
    // None if raw text message, Some(String) for
    //  both JWE and/or JWS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg: Option<String>,
    // Refers to a resource for a set of JSON-encoded public keys, one of
    // which corresponds to the key used to digitally sign the JWS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jku: Option<String>,
    // public key that corresponds to the key used to digitally sign the JWS.
    // TODO: implement proper struct for it:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwk: Option<Jwk>,
    // Some(String) - serialized ephemeral public key.
    // TODO: implement proper struct for it:
    // https://tools.ietf.org/html/draft-looker-jwm-01#section-2.3
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epk: Option<Jwk>,
    // Some("JWM") should be used if nested JWS inside JWE.
    // None otherwise is *STRONGLY RECOMMENDED* by RFC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cty: Option<String>,
}

impl JwmHeader {
    /// Setter of JOSE header properties to identify which signature alg used.
    /// Modifies `typ` and `alg` headers.
    ///
    pub fn as_signed(&mut self, alg: &SignatureAlgorithm) {
        self.typ = String::from("JWM");
        match alg {
            SignatureAlgorithm::EdDsa => {
                self.alg = Some(String::from("EdDSA"));
            },
            SignatureAlgorithm::Es256 => {
                self.alg = Some(String::from("ES256"));
            },
            SignatureAlgorithm::Es256k => {
                self.alg = Some(String::from("ES256K"));
            },
        }
    }
    /// Setter of JOSE header preperties to identify which crypto alg and key type used.
    /// Modifies `enc`, `typ` and `alg` headers.
    ///
    pub fn as_encrypted(&mut self, alg: &CryptoAlgorithm) {
        self.typ = String::from("JWM");
        match alg {
            CryptoAlgorithm::A256GCM => {
                self.enc = Some("A256GCM".into());
                self.alg = Some("ECDH-1PU+A256KW".into());
            },
            CryptoAlgorithm::XC20P => {
                self.enc = Some("XC20P".into());
                self.alg = Some("ECDH-1PU+XC20PKW".into());
            }
        }
    }
    pub fn kid(&mut self, kid: Option<String>) {
        self.kid = kid;
    }
}

impl Default for JwmHeader {
    // Need to make sure nonce is 192 bit long unigue for each message.
    fn default() -> Self {
        JwmHeader {
            typ: "JWM".into(),
            enc: None,
            kid: None,
            skid: None,
            epk: None,
            alg: None,
            cty: None,
            jku: None,
            jwk: None,
        }
    }
}

/// This struct presents single recepient of JWE `recepients` collection.
/// Each recepient should have same body cypher key ecrypted with shared secret.
/// [Spec](https://tools.ietf.org/html/rfc7516#section-7.2.1)
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Recepient {
    pub header: Jwk,
    pub encrypted_key: String
}

impl Recepient {
    pub fn new(header: Jwk, encrypted_key: String) -> Self {
        Recepient {
            header,
            encrypted_key
        }
    }
}
