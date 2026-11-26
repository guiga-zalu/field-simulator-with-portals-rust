use super::util::Scale;

#[derive(Debug, Clone)]
pub enum Region<T>
where
    T: Copy,
{
    Element(T),
    // Subdivision {
    //     scale: Scale,
    //     regions: Regions,
    // },
}

impl<T> Default for Region<T>
where
    T: Copy + Default,
{
    fn default() -> Region<T> {
        Region::Element(Default::default())
    }
}

impl<T> Region<T>
where
    T: Copy,
{
    pub fn element(&self) -> Option<(Scale, T)> {
        match self {
            Region::Element(e) => Some((Scale(1.0), *e)),
            // _ => None,
        }
    }

    pub fn element_mut(&mut self) -> Option<(Scale, &mut T)> {
        match self {
            Region::Element(e) => Some((Scale(1.0), e)),
            // _ => None,
        }
    }

    // pub fn subdivision(&self) -> Option<Regions> {
    //     match self {
    //         Region::Subdivision(s) => Some(s),
    //         _ => None,
    //     }
    // }

    // pub fn count(&self) -> usize {
    //     match self {
    //         Region::Element(_) => 1,
    //         // Region::Subdivision(s) => s.map(|e| e.count()).sum(),
    //     }
    // }
}

pub(crate) type Regions<T> = Vec<Region<T>>;
