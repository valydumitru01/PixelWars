/// Bitmap storage utilities for persisting canvas state.
/// Uses Redis bitmaps for fast pixel read/write with persistence.
use redis::aio::ConnectionManager;
use tracing::info;

const CANVAS_KEY: &str = "pixelwar:canvas:bitmap";

/// Store a pixel color in the Redis bitmap.
pub async fn set_pixel(
    redis: &mut ConnectionManager,
    x: u32,
    y: u32,
    canvas_width: u32,
    color: u32,
) -> anyhow::Result<()> {
    let offset = (y * canvas_width + x) as usize;
    let bytes = color.to_be_bytes();
    // Store 4 bytes (RGBA) at the calculated offset
    redis::cmd("SETRANGE")
        .arg(CANVAS_KEY)
        .arg(offset * 4)
        .arg(&bytes[..])
        .query_async::<()>(redis)
        .await?;
    Ok(())
}

/// Get a pixel color from the Redis bitmap.
pub async fn get_pixel(
    redis: &mut ConnectionManager,
    x: u32,
    y: u32,
    canvas_width: u32,
) -> anyhow::Result<u32> {
    let offset = (y * canvas_width + x) as usize;
    let bytes: Vec<u8> = redis::cmd("GETRANGE")
        .arg(CANVAS_KEY)
        .arg(offset * 4)
        .arg(offset * 4 + 3)
        .query_async(redis)
        .await?;

    if bytes.len() == 4 {
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    } else {
        Ok(0xFFFFFFFF) // default white
    }
}
