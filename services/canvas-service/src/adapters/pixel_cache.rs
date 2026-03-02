use anyhow::Result;
use redis::aio::ConnectionManager;

use crate::ports::PixelCache;
use crate::storage::bitmap;

pub struct RedisPixelCache {
    redis: ConnectionManager,
}

impl RedisPixelCache {
    pub fn new(redis: ConnectionManager) -> Self {
        Self { redis }
    }
}

#[tonic::async_trait]
impl PixelCache for RedisPixelCache {
    async fn set_pixel(&self, x: u32, y: u32, color: u32) -> Result<()> {
        let canvas_width: u32 = 10_000;
        let mut redis = self.redis.clone();
        bitmap::set_pixel(&mut redis, x, y, canvas_width, color).await
    }

    async fn get_pixel(&self, x: u32, y: u32) -> Result<u32> {
        let canvas_width: u32 = 10_000;
        let mut redis = self.redis.clone();
        bitmap::get_pixel(&mut redis, x, y, canvas_width).await
    }

    async fn get_snapshot_region(&self, x: u32, y: u32, width: u32, height: u32) -> Result<Vec<u8>> {
        let canvas_width: u32 = 10_000;
        let mut redis = self.redis.clone();
        let mut data = Vec::with_capacity((width * height * 4) as usize);

        for row in y..(y + height).min(canvas_width) {
            for col in x..(x + width).min(canvas_width) {
                let color = bitmap::get_pixel(&mut redis, col, row, canvas_width).await?;
                data.extend_from_slice(&color.to_be_bytes());
            }
        }

        Ok(data)
    }
}
