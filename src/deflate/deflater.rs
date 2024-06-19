#[cfg(not(feature = "alt-deflater"))]
mod deflater_libdeflater {
    use libdeflater::*;
    use crate::{atomicmin::AtomicMin, PngError, PngResult};

    pub fn deflate(data: &[u8], level: u8, max_size: &AtomicMin) -> PngResult<Vec<u8>> {
        let mut compressor = Compressor::new(CompressionLvl::new(level.into()).unwrap());
        let capacity = max_size
            .get()
            .unwrap_or_else(|| compressor.zlib_compress_bound(data.len()));
        let mut dest = vec![0; capacity];
        let len = compressor
            .zlib_compress(data, &mut dest)
            .map_err(|err| match err {
                CompressionError::InsufficientSpace => PngError::DeflatedDataTooLong(capacity),
            })?;
        dest.truncate(len);
        Ok(dest)
    }

    pub fn inflate(data: &[u8], out_size: usize) -> PngResult<Vec<u8>> {
        let mut decompressor = Decompressor::new();
        let mut dest = vec![0; out_size];
        let len = decompressor
            .zlib_decompress(data, &mut dest)
            .map_err(|err| match err {
                DecompressionError::BadData => PngError::InvalidData,
                DecompressionError::InsufficientSpace => PngError::new("inflated data too long"),
            })?;
        dest.truncate(len);
        Ok(dest)
    }

    pub fn crc32(data: &[u8]) -> u32 {
        let mut crc = Crc::new();
        crc.update(data);
        crc.sum()
    }
}

#[cfg(feature = "alt-deflater")]
mod deflater_flate2 {
    use flate2::{Compression, write::ZlibEncoder, bufread::ZlibDecoder};
    use crc32fast::Hasher;
    use crate::{atomicmin::AtomicMin, PngError, PngResult};
    use std::io::{Read, Write};


    pub fn deflate(data: &[u8], level: u8, max_size: &AtomicMin) -> PngResult<Vec<u8>> {
        let compatible_level = ((level - 1) * 9 + 11) / 11;
        let capacity = max_size.get().unwrap_or_else(|| data.len() * 2);
        let mut encoder = ZlibEncoder::new(Vec::with_capacity(capacity), Compression::new(compatible_level.into()));
        encoder.write_all(data).map_err(|_| PngError::DeflatedDataTooLong(capacity))?;
        let compressed_data = encoder.finish().map_err(|_| PngError::DeflatedDataTooLong(capacity))?;
        Ok(compressed_data)
    }

    pub fn inflate(data: &[u8], out_size: usize) -> PngResult<Vec<u8>> {
        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed_data = vec![0; out_size];
        let len = decoder.read(&mut decompressed_data).map_err(|_| PngError::InvalidData)?;
        decompressed_data.truncate(len);
        Ok(decompressed_data)
    }

    pub fn crc32(data: &[u8]) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize()
    }
}

// Re-export the chosen implementation
#[cfg(not(feature = "alt-deflater"))]
pub use deflater_libdeflater::*;

#[cfg(feature = "alt-deflater")]
pub use deflater_flate2::*;
