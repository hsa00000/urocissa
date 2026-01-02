/// Resize dimensions so that the smaller side equals `target_short_side`, preserving aspect ratio.
///
/// This function ensures that the shortest side of the image is scaled down to `target_short_side`
/// if it exceeds that value. If the shortest side is already smaller than or equal to
/// `target_short_side`, the dimensions remain unchanged.
///
/// # Parameters
/// - `width`: original width of the image.
/// - `height`: original height of the image.
/// - `target_short_side`: the maximum allowed size for the smaller side of the image.
///
/// # Returns
/// A tuple `(new_width, new_height)` representing the scaled dimensions.
pub fn small_width_height(width: u32, height: u32, target_short_side: u32) -> (u32, u32) {
    // Identify the length of the smaller side of the original image
    let min_dimension = std::cmp::min(width, height);

    // Only scale if the smaller side is larger than the target limit
    if min_dimension > target_short_side {
        if width < height {
            // Width is the smaller side (Portrait or Landscape where width < height isn't standard, but logically valid)
            // Scale width to target, calculate height proportionally
            // Formula: new_height = original_height * (target / original_width)
            (target_short_side, height * target_short_side / width)
        } else {
            // Height is the smaller side (Landscape or Square)
            // Scale height to target, calculate width proportionally
            // Formula: new_width = original_width * (target / original_height)
            (width * target_short_side / height, target_short_side)
        }
    } else {
        // The image's smaller side is within the limit, return original dimensions
        (width, height)
    }
}
