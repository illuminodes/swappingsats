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
    let qrcode = fast_qr::QRBuilder::new("https://example.com/")
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
        <div style={size_attr}>
            <div style="width:100%; height:100%;">
                <style>{"
                    div > svg {
                        width: 100%;
                        height: 100%;
                    }
                "}</style>
                { yew::Html::from_html_unchecked(AttrValue::from(svg)) }
            </div>
        </div>
    }
}

// #[derive(Properties, Clone, PartialEq)]
// pub struct BitcoinQrCodeProps {
//     pub id: String,
//     /// Payment methods (at least one must be provided)
//     #[prop_or_default]
//     pub unified: Option<String>,
//     #[prop_or_default]
//     pub bitcoin: Option<String>,
//     #[prop_or_default]
//     pub lightning: Option<String>,
//     #[prop_or_default]
//     pub parameters: Option<String>,
//
//     /// Dimensions and type
//     #[prop_or_default]
//     pub width: Option<String>,
//     #[prop_or_default]
//     pub height: Option<String>,
//     #[prop_or_default]
//     pub type_: Option<QrType>,
//     #[prop_or_default]
//     pub margin: Option<u32>,
//
//     /// Polling and debug options
//     #[prop_or_default]
//     pub is_polling: Option<bool>,
//     #[prop_or_default]
//     pub poll_interval: Option<u32>,
//     #[prop_or_default]
//     pub debug: Option<bool>,
//
//     /// Image options
//     #[prop_or_default]
//     pub image: Option<String>,
//     #[prop_or_default]
//     pub image_embedded: Option<bool>,
//     #[prop_or_default]
//     pub image_hide_background_dots: Option<bool>,
//     #[prop_or_default]
//     pub image_size: Option<f64>,
//     #[prop_or_default]
//     pub image_cross_origin: Option<String>,
//     #[prop_or_default]
//     pub image_margin: Option<u32>,
//
//     /// QR code technical options
//     #[prop_or_default]
//     pub shape: Option<QrShape>,
//     #[prop_or_default]
//     pub qr_type_number: Option<u32>,
//     #[prop_or_default]
//     pub qr_mode: Option<QrMode>,
//     #[prop_or_default]
//     pub qr_error_correction_level: Option<QrErrorCorrectionLevel>,
//
//     /// Style options
//     #[prop_or_default]
//     pub dots_type: Option<QrDotsType>,
//     #[prop_or_default]
//     pub dots_color: Option<String>,
//     #[prop_or_default]
//     pub dots_rotation: Option<f64>,
//     #[prop_or_default]
//     pub corners_square_type: Option<QrCornersSquareType>,
//     #[prop_or_default]
//     pub corners_square_color: Option<String>,
//     #[prop_or_default]
//     pub corners_dot_type: Option<QrCornersDotType>,
//     #[prop_or_default]
//     pub corners_dot_color: Option<String>,
//     #[prop_or_default]
//     pub background_round: Option<u32>,
//     #[prop_or_default]
//     pub background_color: Option<String>,
// }
//
// #[function_component(BitcoinQrCode)]
// pub fn bitcoin_qr(props: &BitcoinQrCodeProps) -> Html {
//     html! {
//         <bitcoin-qr
//             id={props.id.clone()}
//
//             // Payment methods
//             unified={props.unified.clone()}
//             bitcoin={props.bitcoin.clone()}
//             lightning={props.lightning.clone()}
//             parameters={props.parameters.clone()}
//
//             // Dimensions and type
//             width={props.width.clone()}
//             height={props.height.clone()}
//             type={props.type_.as_ref().map(|t| t.as_ref().to_string())}
//             margin={props.margin.map(|m| m.to_string())}
//
//             // Polling and debug options
//             is-polling={props.is_polling.map(|p| p.to_string())}
//             poll-interval={props.poll_interval.map(|p| p.to_string())}
//             debug={props.debug.map(|d| d.to_string())}
//
//             // Image options
//             image={props.image.clone()}
//             image-embedded={props.image_embedded.map(|e| e.to_string())}
//             image-hide-background-dots={props.image_hide_background_dots.map(|h| h.to_string())}
//             image-size={props.image_size.map(|s| s.to_string())}
//             image-cross-origin={props.image_cross_origin.clone()}
//             image-margin={props.image_margin.map(|m| m.to_string())}
//
//             // QR code technical options
//             shape={props.shape.as_ref().map(|s|  s.as_ref().to_string())}
//             qr-type-number={props.qr_type_number.map(|n| n.to_string())}
//             qr-mode={props.qr_mode.as_ref().map(|m|  m.as_ref().to_string())}
//             qr-error-correction-level={props.qr_error_correction_level.as_ref().map(|l| l.as_ref().to_string())}
//
//             // Style options
//             dots-type={props.dots_type.as_ref().map(|t|  t.as_ref().to_string())}
//             dots-color={props.dots_color.clone()}
//             dots-rotation={props.dots_rotation.map(|r| r.to_string())}
//             corners-square-type={props.corners_square_type.as_ref().map(|t| t.as_ref().to_string())}
//             corners-square-color={props.corners_square_color.clone()}
//             corners-dot-type={props.corners_dot_type.as_ref().map(|t| t.as_ref().to_string())}
//             corners-dot-color={props.corners_dot_color.clone()}
//             background-round={props.background_round.map(|r| r.to_string())}
//             background-color={props.background_color.clone()}
//         />
//     }
// }
