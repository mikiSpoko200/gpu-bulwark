// pub mod format;

pub struct Size<const DIM: usize>([usize; DIM]);

impl Size<1> {
    pub fn x(&self) -> usize { 
        self.0[0]
    }
}

impl Size<2> {
    pub fn x(&self) -> usize { 
        self.0[0]
    }

    pub fn y(&self) -> usize { 
        self.0[1]
    }
}

impl Size<3> {
    pub fn x(&self) -> usize { 
        self.0[0]
    }

    pub fn y(&self) -> usize { 
        self.0[1]
    }

    pub fn z(&self) -> usize { 
        self.0[2]
    }
}

pub struct Image<const DIM: usize> {
    size: Size<DIM>,
    
}