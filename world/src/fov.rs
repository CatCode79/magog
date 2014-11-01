use std::iter::{Iterator, Chain};
use std::option::{Item};
use std::mem;
use num::Integer;
use calx::{V2};
use dir6::Dir6;

pub struct Fov<'a> {
    /// Predicate for whether a given point will block the field of view.
    is_opaque: |V2<int>|: 'a -> bool,
    range: uint,
    stack: Vec<Sector>,
}

struct Sector {
    /// Start point of current sector.
    begin: PolarPoint,
    /// Point currently being processed.
    pt: PolarPoint,
    /// End point of current sector.
    end: PolarPoint,
    /// Currently iterating through a sequence of opaque cells.
    group_opaque: bool,
}

/// An iterator that will yield the field of view around the origin V2(0, 0)
/// up to hex grid distance range, with cells for which is_opaque returns true
/// blocking visibility further away in their direction.
impl<'a> Fov<'a> {
    pub fn new<'a>(is_opaque: |V2<int>|: 'a -> bool, range: uint) -> Chain<Item<V2<int>>, Fov<'a>> {
        // The origin position V2(0, 0) is a special case for the traversal
        // algorithm, but it's also always present, so instead of adding ugly
        // branches to the actual iterator, we just chain it in right here.
        let init_group = is_opaque(Dir6::from_int(0).to_v2());
        Some(V2(0i, 0i)).into_iter().chain(Fov {
            is_opaque: is_opaque,
            range: range,
            stack: vec![Sector {
                begin: PolarPoint::new(0.0, 1),
                pt: PolarPoint::new(0.0, 1),
                end: PolarPoint::new(6.0, 1),
                group_opaque: init_group,
            }],
        })
    }
}

impl<'a> Iterator<V2<int>> for Fov<'a> {
    fn next(&mut self) -> Option<V2<int>> {
        if let Some(mut current) = self.stack.pop() {
            if current.pt.is_below(current.end) {
                let pos = current.pt.to_v2();
                let current_opaque = (self.is_opaque)(pos);

                // Terrain opacity changed, branch out.
                if current_opaque != current.group_opaque {
                    // Add the rest of this sector with the new opacity.
                    self.stack.push(Sector {
                        begin: current.pt,
                        pt: current.pt,
                        end: current.end,
                        group_opaque: current_opaque,
                    });

                    // If this was a visible sector and we're below range, branch
                    // out further.
                    if !current.group_opaque && current.begin.radius < self.range {
                        self.stack.push(Sector {
                            begin: current.begin.further(),
                            pt: current.begin.further(),
                            end: current.pt.further(),
                            group_opaque: (self.is_opaque)(current.begin.further().to_v2()),
                        });
                    }
                    return self.next();
                }

                // Otherwise proceed along the current sector.
                current.pt = current.pt.next();
                self.stack.push(current);
                return Some(pos);
            } else {
                // Hit the end of the sector.

                // If this was a visible sector and we're below range, branch
                // out further.
                if !current.group_opaque && current.begin.radius < self.range {
                    self.stack.push(Sector {
                        begin: current.begin.further(),
                        pt: current.begin.further(),
                        end: current.end.further(),
                        group_opaque: (self.is_opaque)(current.begin.further().to_v2()),
                    });
                }

                self.next()
            }
        } else {
            None
        }
    }
}

/// Points on a hex circle expressed in polar coordinates.
#[deriving(PartialEq)]
struct PolarPoint {
    pos: f32,
    radius: uint
}

impl PolarPoint {
    pub fn new(pos: f32, radius: uint) -> PolarPoint { PolarPoint { pos: pos, radius: radius } }
    /// Index of the discrete hex cell along the circle that corresponds to this point.
    fn winding_index(self) -> int { (self.pos + 0.5).floor() as int }

    pub fn is_below(self, other: PolarPoint) -> bool { self.winding_index() < other.end_index() }
    fn end_index(self) -> int { (self.pos + 0.5).ceil() as int }

    pub fn to_v2(self) -> V2<int> {
        if self.radius == 0 { return V2(0, 0); }
        let index = self.winding_index();
        let sector = index.mod_floor(&(self.radius as int * 6)) / self.radius as int;
        let offset = index.mod_floor(&(self.radius as int)) as int;
        let rod = Dir6::from_int(sector).to_v2() * (self.radius as int);
        let tangent = Dir6::from_int((sector + 2) % 6).to_v2() * offset;
        rod + tangent
    }

    /// If this point and the next point are adjacent vertically (along the xy
    /// axis), return a tuple of the point outside of the circle between the
    /// two points.
    ///
    /// This is a helper function for the FOV special case where acute corners
    /// of fake isometric rooms are marked visible even though strict hex FOV
    /// logic would keep them unseen.
    pub fn side_point(self) -> Option<V2<int>> {
        let next = self.next();
        let V2(x1, y1) = self.to_v2();
        let V2(x2, y2) = next.to_v2();

        if x2 == x1 + 1 && y2 == y1 + 1 {
            // Going down the right rim.
            Some(V2(x1 + 1, y1))
        } else if x2 == x1 - 1 && y2 == y1 - 1 {
            // Going up the left rim.
            Some(V2(x1 - 1, y1))
        } else {
            None
        }
    }

    /// The point corresponding to this one on the hex circle with radius +1.
    pub fn further(self) -> PolarPoint {
        PolarPoint::new(
            self.pos * (self.radius + 1) as f32 / self.radius as f32,
            self.radius + 1)
    }

    /// The point next to this one along the hex circle.
    pub fn next(self) -> PolarPoint {
        PolarPoint::new((self.pos + 0.5).floor() + 0.5, self.radius)
    }
}
