use fast_qr::convert::Builder;
use yew::prelude::*;

/// QR Type
///
/// Chooses the HTML element tag type to render the QR code on.
#[derive(Clone, PartialEq, Eq)]
pub enum QrType {
    Canvas,
    Svg,
}
impl AsRef<str> for QrType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Canvas => "canvas",
            Self::Svg => "svg",
        }
    }
}

/// QR Dots Type
///
/// Chooses the style and shape of the QR code dots.
#[derive(Clone, PartialEq, Eq)]
pub enum QrDotsType {
    Square,
    Dots,
    Rounded,
    Classy,
    ClassyRounded,
    ExtraRounded,
}
impl AsRef<str> for QrDotsType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Square => "square",
            Self::Dots => "dots",
            Self::Rounded => "rounded",
            Self::Classy => "classy",
            Self::ClassyRounded => "classy-rounded",
            Self::ExtraRounded => "extra-rounded",
        }
    }
}

/// QR Corners Square type
///
/// Chooses the style of the QR code corners.
#[derive(Clone, PartialEq, Eq)]
pub enum QrCornersSquareType {
    Square,
    ExtraRounded,
    Dot,
}
impl AsRef<str> for QrCornersSquareType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Square => "square",
            Self::ExtraRounded => "extra-rounded",
            Self::Dot => "dot",
        }
    }
}

/// QR Corners Dot Type
///
/// Chooses the style of the QR code corners.
#[derive(Clone, PartialEq, Eq)]
pub enum QrCornersDotType {
    Square,
    Dot,
}
impl AsRef<str> for QrCornersDotType {
    fn as_ref(&self) -> &str {
        match self {
            Self::Square => "square",
            Self::Dot => "dot",
        }
    }
}

/// QR Error Correction Level
///
/// Chooses the level of error correction for the QR code.
#[derive(Clone, PartialEq, Eq)]
pub enum QrErrorCorrectionLevel {
    L,
    M,
    Q,
    H,
}
impl AsRef<str> for QrErrorCorrectionLevel {
    fn as_ref(&self) -> &str {
        match self {
            Self::L => "L",
            Self::M => "M",
            Self::Q => "Q",
            Self::H => "H",
        }
    }
}

/// QR Mode
///
/// Chooses the encoding mode for the QR code.
#[derive(Clone, PartialEq, Eq)]
pub enum QrMode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
}
impl AsRef<str> for QrMode {
    fn as_ref(&self) -> &str {
        match self {
            Self::Numeric => "Numeric",
            Self::Alphanumeric => "Alphanumeric",
            Self::Byte => "Byte",
            Self::Kanji => "Kanji",
        }
    }
}

/// QR Shape
///
/// Chooses the shape of the QR code.
#[derive(Clone, PartialEq, Eq)]
pub enum QrShape {
    Square,
    Circle,
}

impl AsRef<str> for QrShape {
    fn as_ref(&self) -> &str {
        match self {
            Self::Square => "square",
            Self::Circle => "circle",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
pub enum AddressType {
    #[default]
    Bitcoin,
    Lightning,
    Liquid,
}

#[derive(Properties, Clone, PartialEq)]
pub struct BitcoinQrCodeProps {
    pub id: String,
    pub address: String,
    #[prop_or_default]
    pub address_type: AddressType,
    #[prop_or(100)]
    pub size: u32,
    #[prop_or(1)]
    pub margin: u32,
    #[prop_or_default]
    pub image_url: Option<String>,
    #[prop_or_default]
    pub image_size: Option<f64>,
    #[prop_or(fast_qr::convert::Shape::Square)]
    pub dots_shape: fast_qr::convert::Shape,
    #[prop_or("#000000".to_string())]
    pub dots_color: String,
    #[prop_or("#000000".to_string())]
    pub corners_square_color: String,
    #[prop_or("#000000".to_string())]
    pub corners_dot_color: String,
    #[prop_or("#ffffff".to_string())]
    pub background_color: String,
}

#[function_component(BitcoinQrCode)]
pub fn bitcoin_qr(props: &BitcoinQrCodeProps) -> Html {
    let qrcode = fast_qr::QRBuilder::new(props.address.clone())
        .build()
        .unwrap();
    let mut svg = fast_qr::convert::svg::SvgBuilder::default();
    if let Some(image_url) = &props.image_url {
        svg.image(image_url.clone());
    }
    if let Some(image_size) = &props.image_size {
        svg.image_size(*image_size);
    }
    let svg = svg
        .margin(props.margin as usize)
        .shape_color(props.dots_shape, props.dots_color.clone())
        .module_color(props.corners_square_color.clone())
        .background_color(props.background_color.clone())
        .to_str(&qrcode);

    let size_attr = format!(r#"width="{0}px" height="{0}px""#, props.size);

    // Insert the attributes right after <svg

    html! {
        <>
            <style>
                {".qr-wrapper svg { width: 100%; height: 100%; }"}
            </style>
            <div class="qr-wrapper" style={size_attr}>
                { yew::Html::from_html_unchecked(AttrValue::from(svg)) }
            </div>
        </>
    }
}

