use errors::*;
use version::Version;
use tag_value_map::TagValueMap;
use tag::Tag;
use std::convert::TryFrom;
use crypto::proof::Proof;

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

impl<'a> TryFrom<&'a TagValueMap> for ClientHelloMessage {
    type Error = Error;

    fn try_from(value: &'a TagValueMap) -> Result<Self> {
        let server_name = value.get_optional_value(Tag::ServerNameIndication)?;
        let source_address_token = value.get_optional_value(Tag::SourceAddressToken)?;
        let proof_demands = value.get_required_values(Tag::ProofDemand)?;
        let common_certificate_sets = value.get_optional_values(Tag::CommonCertificateSets)?;
        let cached_certificates = value.get_optional_values(Tag::CachedCertificates)?;
        let version = value.get_required_value(Tag::Version)?;
        let leaf_certificate = value.get_required_value(Tag::Fnv1aHash)?;

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
}

impl<'a> From<&'a ClientHelloMessage> for TagValueMap {
    fn from(value: &'a ClientHelloMessage) -> Self {
        let mut tag_value_map = TagValueMap::default();

        if let Some(ref server_name) = value.server_name {
            tag_value_map.set_value(Tag::ServerNameIndication, server_name);
        }

        if let Some(ref source_address_token) = value.source_address_token {
            tag_value_map.set_value(Tag::SourceAddressToken, source_address_token);
        }

        tag_value_map.set_value(Tag::ProofDemand, &value.proof_demands);

        if !value.common_certificate_sets.is_empty() {
            tag_value_map.set_value(Tag::CommonCertificateSets,
                                         &value.common_certificate_sets);
        }

        if !value.cached_certificates.is_empty() {
            tag_value_map.set_value(Tag::CachedCertificates, &value.cached_certificates);
        }

        tag_value_map.set_value(Tag::Version, &value.version);

        tag_value_map.set_value(Tag::Fnv1aHash, &value.leaf_certificate);

        tag_value_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::proof::Proof;
    use tag_value_map::TagValueMap;
    use version::Version;

    #[test]
    fn serializes_to_from_quic_tag_value_map() {
        let chlo = ClientHelloMessage {
            server_name: Some("example.com".to_string()),
            source_address_token: Some(vec![1, 2, 3]),
            proof_demands: vec![Proof::X509],
            common_certificate_sets: vec![5435435, 654123],
            cached_certificates: vec![929080, 7897897],
            version: Version::DRAFT_IETF_01,
            leaf_certificate: 8123678,
        };
        let tag_value_map = TagValueMap::from(&chlo);
        let resultant_chlo = ClientHelloMessage::try_from(&tag_value_map).unwrap();

        assert_eq!(chlo, resultant_chlo);
    }
}