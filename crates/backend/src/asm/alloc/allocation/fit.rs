use common::lir::TypeId;

use super::Allocator;
use crate::asm::RegisterInfo;

impl Allocator<'_> {
    pub fn first_fitting_frame(&self, mut unavailable: Vec<(isize, TypeId)>, ty: TypeId) -> isize {
        unavailable.sort_by(|(off1, _), (off2, _)| off1.cmp(off2));
        let size = isize::try_from(self.types.sizeof(&ty)).unwrap();

        let mut off = match unavailable.get(0) {
            Some((off, ty)) => {
                if *off < 0 || size < *off {
                    return 0;
                } else {
                    off + isize::try_from(self.types.sizeof(ty)).unwrap()
                }
            }
            None => 0,
        };

        for i in 0..(unavailable.len().saturating_sub(1)) {
            let bottom =
                unavailable[i].0 + isize::try_from(self.types.sizeof(&unavailable[i].1)).unwrap();
            let top = unavailable[i + 1].0;
            let gap = top - bottom;
            assert!(gap > 0);

            if gap >= size {
                off = bottom;
                break;
            } else if top >= 0 {
                off = top + isize::try_from(self.types.sizeof(&unavailable[i + 1].1)).unwrap();
            }
        }

        off
    }

    pub fn first_fitting_reg(
        &self,
        possible: &[RegisterInfo],
        unavailable: Vec<usize>,
        ty: TypeId,
    ) -> Option<usize> {
        let size = self.types.sizeof(&ty);
        for reg in possible.iter() {
            if reg.size >= size && !unavailable.contains(&reg.id) {
                return Some(reg.id);
            }
        }

        None
    }
}
