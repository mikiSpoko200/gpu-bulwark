
pub trait AttribFormat {
    const N_COMPONENTS: usize;
}

#[hi::marker]
pub trait Format: AttribFormat { }

#[hi::marker]
pub trait IFormat: AttribFormat { }

#[hi::marker]
pub trait LFormat: AttribFormat { }
