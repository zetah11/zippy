use super::{AllocConstraints, Allocator, Place};

impl<C: AllocConstraints> Allocator<'_, C> {
    pub fn first_fitting_frame(&self, unavailable: Vec<(Place, usize)>, size: usize) -> usize {
        let mut unavailable: Vec<_> = unavailable
            .into_iter()
            .filter_map(|(place, size)| match place {
                Place::Local(local) => Some((local, size)),
                Place::Argument(_) | Place::Parameter(_) => None,
            })
            .collect();

        unavailable.sort_by(|(off1, _), (off2, _)| off1.cmp(off2));

        let mut off = match unavailable.first() {
            Some((off, _)) => {
                if size < *off {
                    return 0;
                } else {
                    (*off).max(0) + size
                }
            }
            None => 0,
        };

        if unavailable.len() > 1 {
            for (other_offset, other_size) in unavailable[1..].iter().copied() {
                let bottom = other_offset.max(0);
                let top = (other_offset + other_size).max(0);
                let gap = top - bottom;

                if gap >= size {
                    off = bottom;
                    break;
                } else {
                    off = top;
                }
            }
        }

        off
    }
}
