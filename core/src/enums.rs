use std::fmt::Display;
use std::fs::File;

use file_format::FileFormat;
use fluent_templates::{LanguageIdentifier, langid};
use serde::{Deserialize, Serialize};

use crate::L10n;

#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[derive(sqlx::Type, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[sqlx(type_name = "activity_action", rename_all = "snake_case")]
pub enum ActivityAction {
    CreateBoard,
    UpdateBoard,
    CreateList,
    UpdateList,
    UpdateListPosition,
    DeleteList,
    CreateCard,
    UpdateCard,
    UpdateCardList,
    UpdateCardPosition,
    DeleteCard,
}

#[allow(clippy::enum_variant_names)]
#[derive(sqlx::Type, Clone, Deserialize, PartialEq, Serialize)]
#[sqlx(type_name = "blob_file_type")]
pub enum BlobFileType {
    #[sqlx(rename = "image/gif")]
    ImageGif,
    #[sqlx(rename = "image/jpeg")]
    ImageJpeg,
    #[sqlx(rename = "image/png")]
    ImagePng,
    #[sqlx(rename = "image/webp")]
    ImageWebp,
}

impl Display for BlobFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlobFileType::ImageGif => write!(f, "image/gif"),
            BlobFileType::ImageJpeg => write!(f, "image/jpeg"),
            BlobFileType::ImagePng => write!(f, "image/png"),
            BlobFileType::ImageWebp => write!(f, "image/webp"),
        }
    }
}

impl TryFrom<&File> for BlobFileType {
    type Error = std::io::Error;

    fn try_from(value: &File) -> Result<Self, Self::Error> {
        let file_format = FileFormat::from_reader(value)?;

        match file_format {
            FileFormat::GraphicsInterchangeFormat => Ok(Self::ImageGif),
            FileFormat::JointPhotographicExpertsGroup => Ok(Self::ImageJpeg),
            FileFormat::PortableNetworkGraphics => Ok(Self::ImagePng),
            FileFormat::Webp => Ok(Self::ImageWebp),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsupported file format",
            )),
        }
    }
}

impl BlobFileType {
    pub fn extension(&self) -> &str {
        match self {
            Self::ImageGif => "gif",
            Self::ImageJpeg => "jpeg",
            Self::ImagePng => "png",
            Self::ImageWebp => "webp",
        }
    }

    pub fn support_thumbnails(&self) -> bool {
        [Self::ImageGif, Self::ImageJpeg, Self::ImagePng, Self::ImageWebp].contains(self)
    }
}

#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[derive(sqlx::Type, Clone, Copy, Deserialize, Eq, Serialize, PartialEq)]
#[sqlx(type_name = "board_visibility", rename_all = "lowercase")]
pub enum BoardVisibility {
    Private,
    Users,
    Public,
}

#[derive(sqlx::Type, Clone, Copy, PartialEq)]
#[sqlx(type_name = "confirmation_action", rename_all = "snake_case")]
pub enum ConfirmationAction {
    Email,
    Login,
    PasswordReset,
}

#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[derive(sqlx::Type, strum::Display, Clone, Copy, Deserialize, Eq, Serialize, PartialEq)]
#[sqlx(type_name = "country_code")]
pub enum CountryCode {
    AF,
    AX,
    AL,
    DZ,
    AS,
    AD,
    AO,
    AI,
    AQ,
    AG,
    AR,
    AM,
    AW,
    AU,
    AT,
    AZ,
    BS,
    BH,
    BD,
    BB,
    BY,
    BE,
    BZ,
    BJ,
    BM,
    BT,
    BO,
    BQ,
    BA,
    BW,
    BV,
    BR,
    IO,
    BN,
    BG,
    BF,
    BI,
    CV,
    KH,
    CM,
    CA,
    KY,
    CF,
    TD,
    CL,
    CN,
    CX,
    CC,
    CO,
    KM,
    CG,
    CD,
    CK,
    CR,
    CI,
    HR,
    CU,
    CW,
    CY,
    CZ,
    DK,
    DJ,
    DM,
    DO,
    EC,
    EG,
    SV,
    GQ,
    ER,
    EE,
    SZ,
    ET,
    FK,
    FO,
    FJ,
    FI,
    FR,
    GF,
    PF,
    TF,
    GA,
    GM,
    GE,
    DE,
    GH,
    GI,
    GR,
    GL,
    GD,
    GP,
    GU,
    GT,
    GG,
    GN,
    GW,
    GY,
    HT,
    HM,
    VA,
    HN,
    HK,
    HU,
    IS,
    IN,
    ID,
    IR,
    IQ,
    IE,
    IM,
    IL,
    IT,
    JM,
    JP,
    JE,
    JO,
    KZ,
    KE,
    KI,
    KP,
    KR,
    KW,
    KG,
    LA,
    LV,
    LB,
    LS,
    LR,
    LY,
    LI,
    LT,
    LU,
    MO,
    MG,
    MW,
    MY,
    MV,
    ML,
    MT,
    MH,
    MQ,
    MR,
    MU,
    YT,
    MX,
    FM,
    MD,
    MC,
    MN,
    ME,
    MS,
    MA,
    MZ,
    MM,
    NA,
    NR,
    NP,
    NL,
    NC,
    NZ,
    NI,
    NE,
    NG,
    NU,
    NF,
    MK,
    MP,
    NO,
    OM,
    PK,
    PW,
    PS,
    PA,
    PG,
    PY,
    PE,
    PH,
    PN,
    PL,
    PT,
    PR,
    QA,
    RE,
    RO,
    RU,
    RW,
    BL,
    SH,
    KN,
    LC,
    MF,
    PM,
    VC,
    WS,
    SM,
    ST,
    SA,
    SN,
    RS,
    SC,
    SL,
    SG,
    SX,
    SK,
    SI,
    SB,
    SO,
    ZA,
    GS,
    SS,
    ES,
    LK,
    SD,
    SR,
    SJ,
    SE,
    CH,
    SY,
    TW,
    TJ,
    TZ,
    TH,
    TL,
    TG,
    TK,
    TO,
    TT,
    TN,
    TR,
    TM,
    TC,
    TV,
    UG,
    UA,
    AE,
    GB,
    US,
    UM,
    UY,
    UZ,
    VU,
    VE,
    VN,
    VG,
    VI,
    WF,
    EH,
    YE,
    ZM,
    ZW,
}

impl CountryCode {
    fn info(&self) -> rust_iso3166::CountryCode {
        rust_iso3166::from_alpha2(&self.to_string()).expect("Could not get country info")
    }

    pub fn name(&self) -> &str {
        self.info().name
    }
}

#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[derive(sqlx::Type, Clone, Copy, Default, Deserialize, Eq, Serialize, PartialEq)]
#[sqlx(type_name = "language_code", rename_all = "lowercase")]
pub enum LanguageCode {
    #[default]
    En,
    Es,
}

impl From<&str> for LanguageCode {
    fn from(value: &str) -> Self {
        let lang = value.split('-').next().unwrap_or(value).to_lowercase();

        match lang.as_str() {
            "es" => Self::Es,
            _ => Self::En,
        }
    }
}

impl LanguageCode {
    pub fn lang_id(&self) -> LanguageIdentifier {
        match self {
            LanguageCode::En => langid!("en"),
            LanguageCode::Es => langid!("es"),
        }
    }

    pub fn to_l10n(&self) -> L10n {
        L10n::from(self)
    }
}
