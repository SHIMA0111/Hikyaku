use std::borrow::Cow;
use std::str::FromStr;
use aws_config::meta::region::ProvideRegion;
use aws_config::{Region as AwsConfigRegion};
use log::error;
use crate::errors::{HikyakuError, HikyakuResult};
use crate::errors::HikyakuError::InvalidArgumentError;
use crate::utils::region::Region;

/// AWSRegion enumerates the various AWS regions.
/// Each variant of the enum represents a specific AWS region, denoted by its common name,
/// with the `get_region` method providing the corresponding region code.
/// (The region based on [Amazon S3 Service Endpoint](https://docs.aws.amazon.com/general/latest/gr/s3.html))
///
/// # Variants
///
/// * `Ohio` - us-east-2
/// * `NVirginia` - us-east-1
/// * `NCalifornia` - us-west-1
/// * `Oregon` - us-west-2
/// * `CapeTown` - af-south-1
/// * `HongKong` - ap-east-1
/// * `Hyderabad` - ap-south-2
/// * `Jakarta` - ap-southeast-3
/// * `Malaysia` - ap-southeast-5
/// * `Melbourne` - ap-southeast-4
/// * `Mumbai` - ap-south-1
/// * `Osaka` - ap-northeast-3
/// * `Seoul` - ap-northeast-2
/// * `Singapore` - ap-southeast-1
/// * `Sydney` - ap-southeast-2
/// * `Tokyo` - ap-northeast-1
/// * `Canada` - ca-central-1
/// * `Calgary` - ca-west-1
/// * `Frankfurt` - eu-central-1
/// * `Ireland` - eu-west-1
/// * `London` - eu-west-2
/// * `Milan` - eu-south-1
/// * `Paris` - eu-west-3
/// * `Spain` - eu-south-2
/// * `Stockholm` - eu-north-1
/// * `Zurich` - eu-central-2
/// * `TelAviv` - il-central-1
/// * `Bahrain` - me-south-1
/// * `UAE` - me-central-1
/// * `SaoPaulo` - sa-east-1
/// * `USEastGovernment` - us-gov-east-1
/// * `USWestGovernment` - us-gov-west-1
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AWSRegion {
    Ohio,
    NVirginia,
    NCalifornia,
    Oregon,
    CapeTown,
    HongKong,
    Hyderabad,
    Jakarta,
    Malaysia,
    Melbourne,
    Mumbai,
    Osaka,
    Seoul,
    Singapore,
    Sydney,
    Tokyo,
    Canada,
    Calgary,
    Frankfurt,
    Ireland,
    London,
    Milan,
    Paris,
    Spain,
    Stockholm,
    Zurich,
    TelAviv,
    Bahrain,
    UAE,
    SaoPaulo,
    USEastGovernment,
    USWestGovernment,
}

impl Region for AWSRegion {
    /// Get the region code from user input region variant.
    fn get_region(&self) -> &str {
        match self {
            AWSRegion::Ohio => "us-east-2",
            AWSRegion::NVirginia => "us-east1",
            AWSRegion::NCalifornia => "us-west-1",
            AWSRegion::Oregon => "us-west-2",
            AWSRegion::CapeTown => "af-south-1",
            AWSRegion::HongKong => "ap-east-1",
            AWSRegion::Hyderabad => "ap-south-2",
            AWSRegion::Jakarta => "ap-southeast-3",
            AWSRegion::Malaysia => "ap-southeast-5",
            AWSRegion::Melbourne => "ap-southeast-4",
            AWSRegion::Mumbai => "ap-south-1",
            AWSRegion::Osaka => "ap-northeast-3",
            AWSRegion::Seoul => "ap-northeast-2",
            AWSRegion::Singapore => "ap-southeast-1",
            AWSRegion::Sydney => "ap-southeast-2",
            AWSRegion::Tokyo => "ap-northeast-1",
            AWSRegion::Canada => "ca-central-1",
            AWSRegion::Calgary => "ca-west-1",
            AWSRegion::Frankfurt => "eu-central-1",
            AWSRegion::Ireland => "eu-west-1",
            AWSRegion::London => "eu-west-2",
            AWSRegion::Milan => "eu-south-1",
            AWSRegion::Paris => "eu-west-3",
            AWSRegion::Spain => "eu-south-2",
            AWSRegion::Stockholm => "eu-north-1",
            AWSRegion::Zurich => "eu-central-2",
            AWSRegion::TelAviv => "il-central-1",
            AWSRegion::Bahrain => "me-south-1",
            AWSRegion::UAE => "me-central-1",
            AWSRegion::SaoPaulo => "sa-east-1",
            AWSRegion::USEastGovernment => "us-gov-east-1",
            AWSRegion::USWestGovernment => "us-gov-west-1",
        }
    }
}

impl FromStr for AWSRegion {
    type Err = HikyakuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        get_aws_region_from_str(s)
    }
}

/// To flexibility, the parser to parse input string to AWSRegion is split from the FromStr implementation.
fn get_aws_region_from_str(region_str: &str) -> HikyakuResult<AWSRegion> {
    let region_str = region_str.to_lowercase();
    
    match region_str.as_str() {
        "us-east-2" | "ohio" => Ok(AWSRegion::Ohio),
        "us-east1" | "virginia" => Ok(AWSRegion::NVirginia),
        "us-west-1" | "california" => Ok(AWSRegion::NCalifornia),
        "us-west-2" | "oregon" => Ok(AWSRegion::Oregon),
        "af-south-1" | "capetown" => Ok(AWSRegion::CapeTown),
        "ap-east-1" | "hongkong" => Ok(AWSRegion::HongKong),
        "ap-south-2" | "hyderabad" => Ok(AWSRegion::Hyderabad),
        "ap-southeast-3" | "jakarta" => Ok(AWSRegion::Jakarta),
        "ap-southeast-5" | "malaysia" => Ok(AWSRegion::Malaysia),
        "ap-southeast-4" | "melbourne" => Ok(AWSRegion::Melbourne),
        "ap-south-1" | "mumbai" => Ok(AWSRegion::Mumbai),
        "ap-northeast-3" | "osaka" => Ok(AWSRegion::Osaka),
        "ap-northeast-2" | "seoul" => Ok(AWSRegion::Seoul),
        "ap-southeast-1" | "singapore" => Ok(AWSRegion::Singapore),
        "ap-southeast-2" | "sydney" => Ok(AWSRegion::Sydney),
        "ap-northeast-1" | "tokyo" => Ok(AWSRegion::Tokyo),
        "ca-central-1" | "canada" => Ok(AWSRegion::Canada),
        "ca-west-1" | "calgary" => Ok(AWSRegion::Calgary),
        "eu-central-1" | "frankfurt" => Ok(AWSRegion::Frankfurt),
        "eu-west-1" | "ireland" => Ok(AWSRegion::Ireland),
        "eu-west-2" | "london" => Ok(AWSRegion::London),
        "eu-south-1" | "milan" => Ok(AWSRegion::Milan),
        "eu-west-3" | "paris" => Ok(AWSRegion::Paris),
        "eu-south-2" | "spain" => Ok(AWSRegion::Spain),
        "eu-north-1" | "stockholm" => Ok(AWSRegion::Stockholm),
        "eu-central-2" | "zurich" => Ok(AWSRegion::Zurich),
        "il-central-1" | "telaviv" => Ok(AWSRegion::TelAviv),
        "me-south-1" | "bahrain" => Ok(AWSRegion::Bahrain),
        "me-central-1" | "uae" => Ok(AWSRegion::UAE),
        "sa-east-1" | "saopaulo" => Ok(AWSRegion::SaoPaulo),
        "us-gov-east-1" => Ok(AWSRegion::USEastGovernment),
        "us-gov-west-1" => Ok(AWSRegion::USWestGovernment),
        _ => {
            error!("{} not exist in AWS region", region_str);
            Err(InvalidArgumentError(format!("{} not exist in AWS region", region_str)))
        }
    }
}

impl ProvideRegion for AWSRegion {
    fn region(&self) -> aws_config::meta::region::future::ProvideRegion {
        aws_config::meta::region::future::ProvideRegion::new(async { 
            Some(AwsConfigRegion::new(self.get_region().to_string()))
        })
    }
}

impl TryFrom<AwsConfigRegion> for AWSRegion {
    type Error = HikyakuError;
    
    fn try_from(value: AwsConfigRegion) -> Result<Self, Self::Error> {
        get_aws_region_from_str(value.as_ref())
    }
}

impl Default for AWSRegion {
    fn default() -> Self {
        AWSRegion::Ohio
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::utils::region::Region;
    use super::AWSRegion;
    
    const AWS_REGION: [(&str, AWSRegion, &str); 32] = [
        ("ohio", AWSRegion::Ohio, "us-east-2"),
        ("virginia", AWSRegion::NVirginia, "us-east1"),
        ("california", AWSRegion::NCalifornia, "us-west-1"),
        ("oregon", AWSRegion::Oregon, "us-west-2"),
        ("capetown", AWSRegion::CapeTown, "af-south-1"),
        ("hongkong", AWSRegion::HongKong, "ap-east-1"),
        ("hyderabad", AWSRegion::Hyderabad, "ap-south-2"),
        ("jakarta", AWSRegion::Jakarta, "ap-southeast-3"),
        ("malaysia", AWSRegion::Malaysia, "ap-southeast-5"),
        ("melbourne", AWSRegion::Melbourne, "ap-southeast-4"),
        ("mumbai", AWSRegion::Mumbai, "ap-south-1"),
        ("osaka", AWSRegion::Osaka, "ap-northeast-3"),
        ("seoul", AWSRegion::Seoul, "ap-northeast-2"),
        ("singapore", AWSRegion::Singapore, "ap-southeast-1"),
        ("sydney", AWSRegion::Sydney, "ap-southeast-2"),
        ("tokyo", AWSRegion::Tokyo, "ap-northeast-1"),
        ("canada", AWSRegion::Canada, "ca-central-1"),
        ("calgary", AWSRegion::Calgary, "ca-west-1"),
        ("frankfurt", AWSRegion::Frankfurt, "eu-central-1"),
        ("ireland", AWSRegion::Ireland, "eu-west-1"),
        ("london", AWSRegion::London, "eu-west-2"),
        ("milan", AWSRegion::Milan, "eu-south-1"),
        ("paris", AWSRegion::Paris, "eu-west-3"),
        ("spain", AWSRegion::Spain, "eu-south-2"),
        ("stockholm", AWSRegion::Stockholm, "eu-north-1"),
        ("zurich", AWSRegion::Zurich, "eu-central-2"),
        ("telaviv", AWSRegion::TelAviv, "il-central-1"),
        ("bahrain", AWSRegion::Bahrain, "me-south-1"),
        ("uae", AWSRegion::UAE, "me-central-1"),
        ("saopaulo", AWSRegion::SaoPaulo, "sa-east-1"),
        ("us-gov-east-1", AWSRegion::USEastGovernment, "us-gov-east-1"),
        ("us-gov-west-1", AWSRegion::USWestGovernment, "us-gov-west-1"),
    ];

    #[test]
    fn test_region_valid_inputs() {
        for (region_str, region, region_id) in AWS_REGION {
            assert_eq!(region.get_region(), region_id);
            let region_from_str = AWSRegion::from_str(region_str).unwrap();
            assert_eq!(region, region_from_str);
            let region_from_id = AWSRegion::from_str(region_id).unwrap();
            assert_eq!(region, region_from_id);
        }
    }
    
    #[test]
    fn test_region_invalid_inputs() {
        let region_str = "no-exist-1";
        let region_from_str = AWSRegion::from_str(region_str);
        assert!(region_from_str.is_err());
        assert_eq!(region_from_str.unwrap_err().to_string(), format!("Get invalid argument error: {} not exist in AWS region", region_str));
    }

    #[test]
    fn test_region_default() {
        let region = AWSRegion::default();
        assert_eq!(region.get_region(), "us-east-2");
    }
}
