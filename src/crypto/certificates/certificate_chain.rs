use crypto::certificates::Certificate;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CertificateChain {
    certificates: Vec<Certificate>,
}

impl CertificateChain {
    pub fn leaf_certificate(&self) -> Option<&Certificate> {
        self.certificates.first()
    }

    pub fn intermediate_certificates(&self) -> &[Certificate] {
        &self.certificates
             .split_first()
             .map(|t| t.1)
             .unwrap_or(&[])
    }

    pub fn certificates(&self) -> &[Certificate] {
        self.certificates.as_slice()
    }
}

impl From<Vec<Certificate>> for CertificateChain {
    fn from(value: Vec<Certificate>) -> Self {
        CertificateChain { certificates: value }
    }
}

impl From<CertificateChain> for Vec<Certificate> {
    fn from(value: CertificateChain) -> Vec<Certificate> {
        value.certificates
    }
}
