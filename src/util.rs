use std::default::{Default};
use std::cmp::{min, max};
use std::num::{zero};
use image::{GenericImage, Pixel, ImageBuf, Rgba};
use geom::{V2, Rect};
use rgb::Rgb;

/// Set alpha channel to transparent if pixels have a specific color.
pub fn color_key<P: Pixel<u8>, I: GenericImage<P>>(
    image: &I, color: &Rgb) -> ImageBuf<Rgba<u8>> {
    let (w, h) = image.dimensions();
    let pixels = image.pixels().map(|(_, _, p)| {
            let (pr, pg, pb, mut pa) = p.channels4();
            if pr == color.r && pg == color.g && pb == color.b {
                pa = Default::default();
            }
            Pixel::from_channels(pr, pg, pb, pa)
        }).collect();
    ImageBuf::from_pixels(pixels, w, h)
}

/// Return the rectangle enclosing the parts of the image that aren't fully
/// transparent.
pub fn crop_alpha<T: Primitive+Default, P: Pixel<T>, I: GenericImage<P>>(
    image: &I) -> Rect<int> {
    let (w, h) = image.dimensions();
    let mut p1 = V2(w as int, h as int);
    let mut p2 = V2(0i, 0i);
    let transparent: T = Default::default();
    for y in range(0, h as int) {
        for x in range(0, w as int) {
            let (_, _, _, a) = image.get_pixel(x as u32, y as u32).channels4();
            if a != transparent {
                p1.0 = min(x, p1.0);
                p2.0 = max(x + 1, p2.0);
                p1.1 = min(y, p1.1);
                p2.1 = max(y + 1, p2.1);
            }
        }
    }

    if p1.0 > p2.0 { Rect(V2(0, 0), V2(0, 0)) } // Empty image.
    else { Rect(p1, p2 - p1) }
}

pub fn blit<T: Primitive+Default, P: Pixel<T>, I: GenericImage<P>, J: GenericImage<P>> (
    image: &I, target: &mut J, offset: V2<int>) {
    let (w, h) = image.dimensions();
    // TODO: Check for going over bounds.
    for y in range(0, h) {
        for x in range(0, w) {
            target.put_pixel(x + offset.0 as u32, y + offset.1 as u32, image.get_pixel(x, y));
        }
    }
}

/// Try to pack several small rectangles into one large rectangle. Return
/// offsets for the subrectangles within the container if a packing was found.
pub fn pack_rectangles<T: Primitive+Ord+Clone>(
    container_dim: V2<T>,
    dims: &Vec<V2<T>>)
    -> Option<Vec<V2<T>>> {
    let total_area = dims.iter().map(|dim| dim.0 * dim.1).fold(zero::<T>(), |a, b| a + b);

    // Too much rectangle area to fit in container no matter how you pack it.
    // Fail early.
    if total_area > container_dim.0 * container_dim.1 { return None; }

    // Take enumeration to keep the original indices around.
    let mut largest_first : Vec<(uint, &V2<T>)> = dims.iter().enumerate().collect();
    largest_first.sort_by(|&(_i, a), &(_j, b)| (b.0 * b.1).cmp(&(a.0 * a.1)));

    let mut slots = vec![Rect(V2(zero::<T>(), zero::<T>()), container_dim)];

    let mut ret = Vec::from_elem(dims.len(), V2(zero::<T>(), zero::<T>()));

    for i in range(0, largest_first.len()) {
        let (idx, &dim) = largest_first[i];
        match place(dim, &mut slots) {
            Some(pos) => { *ret.get_mut(idx) = pos; }
            None => { return None; }
        }
    }

    return Some(ret);

    ////////////////////////////////////////////////////////////////////////

    /// Find the smallest slot in the slot vector that will fit the given
    /// item.
    fn place<T: Primitive+Ord>(
        dim: V2<T>, slots: &mut Vec<Rect<T>>) -> Option<V2<T>> {
        for i in range(0, slots.len()) {
            // XXX: Can't use [] indexing on slot because it's mut.
            let &Rect(slot_pos, slot_dim) = slots.get_mut(i);
            if fits(dim, slot_dim) {
                // Remove the original slot, it gets the item. Add the two new
                // rectangles that form around the item.
                let (new_1, new_2) = remaining_rects(dim, Rect(slot_pos, slot_dim));
                slots.swap_remove(i);
                slots.push(new_1);
                slots.push(new_2);
                // Sort by area from smallest to largest.
                slots.sort_by(|&a, &b| a.area().cmp(&b.area()));
                return Some(slot_pos);
            }
        }
        None
    }

    /// Return the two remaining parts of container rect when the dim-sized
    /// item is placed in the top left corner.
    fn remaining_rects<T: Primitive+Ord>(
        dim: V2<T>, Rect(rect_pos, rect_dim): Rect<T>) ->
        (Rect<T>, Rect<T>) {
        assert!(fits(dim, rect_dim));

        // Choose between making a vertical or a horizontal split
        // based on which leaves a bigger open rectangle.
        let vert_vol = max(rect_dim.0 * (rect_dim.1 - dim.1),
            (rect_dim.0 - dim.0) * dim.1);
        let horiz_vol = max(dim.0 * (rect_dim.1 - dim.1),
            (rect_dim.0 - dim.0) * rect_dim.1);

        if vert_vol > horiz_vol {
            //     |AA
            // ----+--
            // BBBBBBB
            // BBBBBBB
            (Rect(V2(rect_pos.0 + dim.0, rect_pos.1), V2(rect_dim.0 - dim.0, dim.1)),
             Rect(V2(rect_pos.0, rect_pos.1 + dim.1), V2(rect_dim.0, rect_dim.1 - dim.1)))
        } else {
            //     |BB
            // ----+BB
            // AAAA|BB
            // AAAA|BB
            (Rect(V2(rect_pos.0, rect_pos.1 + dim.1), V2(dim.0, rect_dim.1 - dim.1)),
             Rect(V2(rect_pos.0 + dim.0, rect_pos.1), V2(rect_dim.0 - dim.0, rect_dim.1)))
        }
    }

    fn fits<T: Ord>(dim: V2<T>, container_dim: V2<T>) -> bool {
        dim.0 <= container_dim.0 && dim.1 <= container_dim.1
    }
}

/*
#[cfg(test)]
mod tests {
    use std::fmt::Show;
    use image::{Rgba, ImageBuf, GenericImage};
    use util;

    #[test]
    fn test_zero_atlas() {
        let empty: Vec<ImageBuf<Rgba<u8>>> = vec![];
        let (canvas, rects) = util::build_atlas(&empty);
        assert!(rects.len() == 0);
    }

    #[test]
    fn test_atlas() {
        let images: Vec<ImageBuf<Rgba<u8>>> = vec![
            ImageBuf::new(6, 4),
            ImageBuf::new(8, 10),
            ImageBuf::new(3, 3),
        ];
        let (canvas, rects) = util::build_atlas(&images);
        assert!(rects.len() == images.len());
        assert!(canvas.dimensions() == (16, 16));
        // Filling the rect starts from top left, and the biggest item goes
        // first.
        rects_equal(&rects.1, &([0, 0], [8, 10]));
        // The next rect goes below the first one since there is a smaller
        // space there.
        rects_equal(&rects.0, &([0, 10], [6, 4]));
        rects_equal(&rects[2], &([8, 0], [3, 3]));
    }

    // XXX: can't compare fixed arrays naively.
    fn rects_equal<T: Primitive+Show>(
        &(ref p1, ref d1): &(V2<T>, V2<T>),
        &(ref p2, ref d2): &(V2<T>, V2<T>)) {
        let (p1, p2, d1, d2) = (
            p1.as_slice(), p2.as_slice(),
            d1.as_slice(), d2.as_slice());
        println!("{}, {}", p1, p2);
        println!("{}, {}", d1, d2);
        assert!(p1 == p2);
        assert!(d1 == d2);
    }
}
*/
