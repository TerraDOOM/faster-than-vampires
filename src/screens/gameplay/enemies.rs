




#[repr(usize)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Type {
    Flagship,
    EmpireGoon,
    PirateShip,
    Outpoust,
}

#[derive(Resource)]
pub struct Ship {
    pub type: Type,
    pub position : (f32,f32),
    pub lifetime : f32,
    pub weapons :
}
