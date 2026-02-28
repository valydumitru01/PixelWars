use shared_common::models::parcel::CANVAS_SIZE;
use std::collections::HashSet;

/// Verify that a set of pixel coordinates forms a topologically contiguous region.
/// Uses flood-fill (BFS) from the first pixel.
pub fn is_contiguous(pixels: &[(u32, u32)]) -> bool {
    if pixels.is_empty() {
        return false;
    }

    let pixel_set: HashSet<(u32, u32)> = pixels.iter().cloned().collect();
    let mut visited: HashSet<(u32, u32)> = HashSet::new();
    let mut queue = std::collections::VecDeque::new();

    queue.push_back(pixels[0]);
    visited.insert(pixels[0]);

    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in &[(0i32, 1i32), (0, -1), (1, 0), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as u32) < CANVAS_SIZE && (ny as u32) < CANVAS_SIZE {
                let neighbor = (nx as u32, ny as u32);
                if pixel_set.contains(&neighbor) && !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }
    }

    visited.len() == pixel_set.len()
}

/// Verify a rectangular region is valid.
pub fn is_valid_rectangle(origin_x: u32, origin_y: u32, width: u32, height: u32) -> bool {
    width > 0
        && height > 0
        && width * height == shared_common::models::parcel::PARCEL_PIXEL_COUNT
        && origin_x + width <= CANVAS_SIZE
        && origin_y + height <= CANVAS_SIZE
}
