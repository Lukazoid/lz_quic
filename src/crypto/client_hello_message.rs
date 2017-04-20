use errors::*;
use quic_version::QuicVersion;
use quic_tag_value_map::QuicTagValueMap;
use quic_tag::QuicTag;
use std::convert::TryFrom;
use crypto::proof::Proof;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClientHelloMessage {
    pub server_name: Option<String>,
    pub source_address_token: Option<Vec<u8>>,
    pub proof_demands: Vec<Proof>,
    pub common_certificate_sets: Vec<u64>,
    pub cached_certificates: Vec<u64>,
    pub version: QuicVersion,
    pub leaf_certificate: u64,
}

impl<'a> TryFrom<&'a QuicTagValueMap> for ClientHelloMessage {
    type Error = Error;

    fn try_from(value: &'a QuicTagValueMap) -> Result<Self> {
        let server_name = value.get_optional_value(QuicTag::ServerNameIndication)?;
        let source_address_token = value.get_optional_value(QuicTag::SourceAddressToken)?;
        let proof_demands = value.get_required_values(QuicTag::ProofDemand)?;
        let common_certificate_sets = value.get_optional_values(QuicTag::CommonCertificateSets)?;
        let cached_certificates = value.get_optional_values(QuicTag::CachedCertificates)?;
        let version = value.get_required_value(QuicTag::Version)?;
        let leaf_certificate = value.get_required_value(QuicTag::Fnv1aHash)?;

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

impl<'a> From<&'a ClientHelloMessage> for QuicTagValueMap {
    fn from(value: &'a ClientHelloMessage) -> Self {
        let mut quic_tag_value_map = QuicTagValueMap::default();

        if let Some(ref server_name) = value.server_name {
            quic_tag_value_map.set_value(QuicTag::ServerNameIndication, server_name);
        }

        if let Some(ref source_address_token) = value.source_address_token {
            quic_tag_value_map.set_value(QuicTag::SourceAddressToken, source_address_token);
        }

        quic_tag_value_map.set_value(QuicTag::ProofDemand, &value.proof_demands);

        if !value.common_certificate_sets.is_empty() {
            quic_tag_value_map.set_value(QuicTag::CommonCertificateSets,
                                         &value.common_certificate_sets);
        }

        if !value.cached_certificates.is_empty() {
            quic_tag_value_map.set_value(QuicTag::CachedCertificates, &value.cached_certificates);
        }

        quic_tag_value_map.set_value(QuicTag::Version, &value.version);

        quic_tag_value_map.set_value(QuicTag::Fnv1aHash, &value.leaf_certificate);

        quic_tag_value_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::proof::Proof;
    use quic_tag_value_map::QuicTagValueMap;
    use quic_version::QuicVersion;

    #[test]
    fn serializes_to_from_quic_tag_value_map() {
        let chlo = ClientHelloMessage {
            server_name: Some("example.com".to_string()),
            source_address_token: Some(vec![1, 2, 3]),
            proof_demands: vec![Proof::X509],
            common_certificate_sets: vec![5435435, 654123],
            cached_certificates: vec![929080, 7897897],
            version: QuicVersion::DRAFT_IETF_01,
            leaf_certificate: 8123678,
        };
        let quic_tag_value_map = QuicTagValueMap::from(&chlo);
        let resultant_chlo = ClientHelloMessage::try_from(&quic_tag_value_map).unwrap();

        assert_eq!(chlo, resultant_chlo);
    }
}