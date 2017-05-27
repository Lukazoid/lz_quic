#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TrustAnchor {
    pub subject: Vec<u8>,
    pub spki: Vec<u8>,
    pub name_constraints: Option<Vec<u8>>,
}

impl TrustAnchor {
    pub fn from_webpki_trust_anchor(trust_anchor: &::webpki::TrustAnchor) -> Self {
        TrustAnchor {
            subject: trust_anchor.subject.to_vec(),
            spki: trust_anchor.spki.to_vec(),
            name_constraints: trust_anchor
                .name_constraints
                .as_ref()
                .map(|x| x.to_vec()),
        }
    }
    pub fn as_webpki_trust_anchor(&self) -> ::webpki::TrustAnchor {
        ::webpki::TrustAnchor {
            subject: &self.subject,
            spki: &self.spki,
            name_constraints: self.name_constraints.as_ref().map(|v| v.as_slice()),
        }
    }
}

