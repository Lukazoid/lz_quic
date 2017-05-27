use errors::*;
use protocol::Version;
use handshake::{Tag, TagValueMap};
use conv::TryFrom;
use crypto::Proof;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClientHelloMessage {
    pub server_name: Option<String>,
    pub source_address_token: Option<Vec<u8>>,
    pub proof_demands: Vec<Proof>,
    pub common_certificate_sets: Vec<u64>,
    pub cached_certificates: Vec<u64>,
    pub version: Version,
    pub leaf_certificate: u64,
}

impl ClientHelloMessage {
    pub fn from_tag_value_map(tag_value_map: &TagValueMap) -> Result<Self> {
        let server_name = tag_value_map
            .get_optional_value(Tag::ServerNameIndication)?;
        let source_address_token = tag_value_map
            .get_optional_value(Tag::SourceAddressToken)?;
        let proof_demands = tag_value_map.get_required_values(Tag::ProofDemand)?;
        let common_certificate_sets = tag_value_map
            .get_optional_values(Tag::CommonCertificateSets)?;
        let cached_certificates = tag_value_map
            .get_optional_values(Tag::CachedCertificates)?;
        let version = tag_value_map.get_required_value(Tag::Version)?;
        let leaf_certificate = tag_value_map.get_required_value(Tag::Fnv1aHash)?;

        Ok(ClientHelloMessage {
               server_name: server_name,
               source_address_token: source_address_token,
               proof_demands: proof_demands,
               common_certificate_sets: common_certificate_sets,
               cached_certificates: cached_certificates,
               version: version,
               leaf_certificate: leaf_certificate,
           })
    }

    pub fn to_tag_value_map(&self) -> TagValueMap {
        let mut tag_value_map = TagValueMap::default();

        if let Some(ref server_name) = self.server_name {
            tag_value_map.set_value(Tag::ServerNameIndication, server_name);
        }

        if let Some(ref source_address_token) = self.source_address_token {
            tag_value_map.set_value(Tag::SourceAddressToken, source_address_token);
        }

        tag_value_map.set_value(Tag::ProofDemand, &self.proof_demands);

        if !self.common_certificate_sets.is_empty() {
            tag_value_map.set_value(Tag::CommonCertificateSets, &self.common_certificate_sets);
        }

        if !self.cached_certificates.is_empty() {
            tag_value_map.set_value(Tag::CachedCertificates, &self.cached_certificates);
        }

        tag_value_map.set_value(Tag::Version, &self.version);

        tag_value_map.set_value(Tag::Fnv1aHash, &self.leaf_certificate);

        tag_value_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::Proof;
    use handshake::TagValueMap;
    use protocol::version;

    #[test]
    fn serializes_to_from_quic_tag_value_map() {
        let chlo = ClientHelloMessage {
            server_name: Some("example.com".to_owned()),
            source_address_token: Some(vec![1, 2, 3]),
            proof_demands: vec![Proof::X509],
            common_certificate_sets: vec![5435435, 654123],
            cached_certificates: vec![929080, 7897897],
            version: version::DRAFT_IETF_01,
            leaf_certificate: 8123678,
        };
        let tag_value_map = chlo.to_tag_value_map();
        let resultant_chlo = ClientHelloMessage::from_tag_value_map(&tag_value_map).unwrap();

        assert_eq!(chlo, resultant_chlo);
    }
}

