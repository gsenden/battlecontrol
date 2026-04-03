#[derive(Clone, Copy)]
pub struct ShipInput {
    pub left: bool,
    pub right: bool,
    pub thrust: bool,
    pub weapon: bool,
    pub special: bool,
}
