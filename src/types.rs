#[derive(Debug, Clone, PartialEq, Default)]
pub struct NumberFormat {
    pub fill_zeros: bool,
    pub digits: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FloatFormat {
    pub base: NumberFormat,
    pub fraction: NumberFormat,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct HexFormat {
    pub uppercase: bool,
    pub nf: NumberFormat,
}
