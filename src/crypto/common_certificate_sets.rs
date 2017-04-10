use crypto::certificate_set::CertificateSet;
use crypto::common_certificate_set_2;
use crypto::common_certificate_set_3;

pub fn build_common_certificate_sets() -> Vec<CertificateSet> {
    vec![common_certificate_set_2::build_common_certificate_set_2(),
         common_certificate_set_3::build_common_certificate_set_3()]
}